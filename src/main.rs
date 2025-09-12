mod config;
mod handler_trait;
mod handlers;
mod logging_middleware;
mod middleware_chain;
mod middleware_trait;
mod simple_handler;

use config::load_config;
use flexi_logger::{Cleanup, Criterion, FileSpec, Logger, Naming, WriteMode};
use handler_trait::Handler;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use log::info;
use logging_middleware::LoggingMiddleware;
use simple_handler::SimpleHandler;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Arc::new(load_config("config.yaml"));
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let access_log_path = config
        .access_log
        .clone()
        .unwrap_or_else(|| format!("log/{}-access.log", today));
    let _error_log_path = config
        .error_log
        .clone()
        .unwrap_or_else(|| format!("log/{}-error.log", today));

    Logger::try_with_str("info")?
        .log_to_file(FileSpec::try_from(access_log_path)?)
        .write_mode(WriteMode::BufferAndFlush)
        .rotate(
            Criterion::Age(flexi_logger::Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(30),
        )
        .format(|w, now, record| {
            // Format mirip Nginx: [time] LEVEL target: message
            write!(
                w,
                "{} [{}] {}: {}\n",
                now.now().format("%d/%b/%Y:%H:%M:%S %z"),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .start()?;
    // TODO: Untuk error log, bisa gunakan log crate dengan target khusus dan Logger kedua jika ingin benar-benar terpisah.

    // config sudah di-load di atas
    info!("Loaded config: {:?}", config);
    if let Some(ref dir) = config.static_dir {
        info!("Static files will be served from: {}", dir);
    }
    if let Some(ref proxy) = config.proxy_pass {
        info!("Proxying requests to: {}", proxy);
    }

    let addr = SocketAddr::new(config.address.parse()?, config.port);
    let listener = TcpListener::bind(addr).await?;
    info!("Server running on http://{}", addr);

    // Build handler and middleware chain using builder
    let handler: Arc<dyn Handler> = Arc::new(SimpleHandler);
    let logging_middleware = Arc::new(LoggingMiddleware::new());
    let chain = middleware_chain::MiddlewareChainBuilder::new()
        .add_middleware(logging_middleware)
        .build(handler);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let config = config.clone();
        let chain = chain.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
                        let config = config.clone();
                        let chain = chain.clone();
                        async move { chain.handle(req, config).await }
                    }),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
