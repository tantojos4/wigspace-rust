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
mod modules {
    pub mod dynamic_loader;
}
use modules::dynamic_loader::{CAbiModule, DynamicModule};
use std::net::SocketAddr;
use std::path::PathBuf;
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

    // --- PLUGIN LOADER ---
    // Load the plugin_example .so at startup
    let mut so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    so_path.push("src/modules/plugin_example/target/release/libplugin_example.so");
    let plugin: Option<CAbiModule> = if so_path.exists() {
        // SAFETY: We trust the plugin to follow the C ABI contract
        match unsafe { CAbiModule::load(&so_path) } {
            Ok(m) => Some(m),
            Err(e) => {
                eprintln!("Failed to load plugin: {}", e);
                None
            }
        }
    } else {
        eprintln!("Plugin .so not found: {}", so_path.display());
        None
    };
    let plugin: Arc<Option<CAbiModule>> = Arc::new(plugin);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let config = config.clone();
        let chain = chain.clone();
        let plugin = plugin.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
                        let config = config.clone();
                        let chain = chain.clone();
                        let plugin = plugin.clone();
                        async move {
                            // If the request is to /plugin, delegate to the plugin
                            if req.uri().path() == "/plugin" {
                                if let Some(plugin) = plugin.as_ref() {
                                    let input = format!("{} {}", req.method(), req.uri());
                                    let output = plugin.handle(&input);
                                    let resp = hyper::Response::new(http_body_util::Full::new(
                                        hyper::body::Bytes::from(output),
                                    ));
                                    Ok::<_, std::convert::Infallible>(resp)
                                } else {
                                    let resp = hyper::Response::builder()
                                        .status(500)
                                        .body(http_body_util::Full::new(hyper::body::Bytes::from(
                                            "Plugin not loaded",
                                        )))
                                        .unwrap();
                                    Ok::<_, std::convert::Infallible>(resp)
                                }
                            } else {
                                chain.handle(req, config).await
                            }
                        }
                    }),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
