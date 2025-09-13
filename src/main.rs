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
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use simple_handler::SimpleHandler;
use std::sync::RwLock;
mod modules {
    pub mod dynamic_loader;
}
use modules::dynamic_loader::{
    CAbiModule, DynamicModule, PluginLifecycle, RustDylibModule, ScriptingModule, WasmModule,
};

#[derive(Clone)]
enum PluginInstance {
    CAbi(Arc<CAbiModule>),
    Lua(Arc<ScriptingModule>),
    Wasm(Arc<WasmModule>),
}
// Load WASM plugin at startup
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // --- LOGGING INIT FIRST ---
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    // Ensure log directory exists
    std::fs::create_dir_all("log").expect("Failed to create log directory");
    let temp_config = load_config("config.yaml");
    let access_log_spec = FileSpec::default()
        .directory("log")
        .basename(format!("access_r{}", today));
    let _error_log_path = temp_config
        .error_log
        .clone()
        .unwrap_or_else(|| format!("log/error_r{}.log", today));

    Logger::try_with_str("info")?
        .log_to_file(access_log_spec)
        .write_mode(WriteMode::BufferAndFlush)
        .rotate(
            Criterion::Age(flexi_logger::Age::Day),
            Naming::Numbers,
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
    // --- END LOGGING INIT ---

    let config = Arc::new(RwLock::new(temp_config));

    {
        let config_read = config.read().unwrap();
        info!("Loaded config: {:?}", *config_read);
        if let Some(ref dir) = config_read.static_dir {
            info!("Static files will be served from: {}", dir);
        }
        if let Some(ref proxy) = config_read.proxy_pass {
            info!("Proxying requests to: {}", proxy);
        }
        if let Some(ref plugins_dir) = config_read.plugins_dir {
            info!("Plugins directory: {}", plugins_dir);
        }
    }

    // --- HOT-RELOAD CONFIG ---
    let config_watcher = config.clone();
    std::thread::spawn(move || {
        log::info!("[hot-reload] watcher thread started");
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        log::info!(
                            "[hot-reload] notify event: kind={:?} paths={:?}",
                            event.kind,
                            event.paths
                        );
                        // Reload config on any event for now
                        let new_config = load_config("config.yaml");
                        let mut w = config_watcher.write().unwrap();
                        *w = new_config;
                        log::info!("[hot-reload] config.yaml reloaded");
                    }
                    Err(e) => {
                        log::error!("[hot-reload] Watch error: {:?}", e);
                    }
                }
            },
            notify::Config::default(),
        )
        .unwrap();
        watcher
            .watch(
                std::path::Path::new("config.yaml"),
                RecursiveMode::Recursive,
            )
            .unwrap();
        // Keep thread alive
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3600));
        }
    });

    let config_read = config.read().unwrap();
    let addr = SocketAddr::new(config_read.address.parse()?, config_read.port);
    let listener = TcpListener::bind(addr).await?;
    info!("Server running on http://{}", addr);

    // Build handler and middleware chain using builder
    let handler: Arc<dyn Handler> = Arc::new(SimpleHandler);
    let logging_middleware = Arc::new(LoggingMiddleware::new());
    let chain = middleware_chain::MiddlewareChainBuilder::new()
        .add_middleware(logging_middleware)
        .build(handler);

    // --- DYNAMIC PLUGIN LOADER & ENDPOINT MAPPING ---
    use std::collections::HashMap;
    let plugins_dir = config_read.plugins_dir.clone().unwrap_or_else(|| "./plugins".to_string());
    let mut endpoint_plugins: HashMap<String, PluginInstance> = HashMap::new();
    let mut loaded_plugins_log = Vec::new();
    if let Some(ref mapping) = config_read.plugin_endpoints {
        for (endpoint, filename) in mapping.iter() {
            let path = PathBuf::from(&plugins_dir).join(filename);
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            match ext {
                "so" => {
                    match unsafe { CAbiModule::load(&path) } {
                        Ok(m) => {
                            endpoint_plugins.insert(endpoint.clone(), PluginInstance::CAbi(Arc::new(m)));
                            loaded_plugins_log.push(format!("{} -> {} [CAbi]", endpoint, filename));
                        },
                        Err(e) => {
                            eprintln!("Failed to load C ABI plugin {}: {}", path.display(), e);
                        }
                    }
                },
                "lua" => {
                    match ScriptingModule::load(&path) {
                        Ok(m) => {
                            endpoint_plugins.insert(endpoint.clone(), PluginInstance::Lua(Arc::new(m)));
                            loaded_plugins_log.push(format!("{} -> {} [Lua]", endpoint, filename));
                        },
                        Err(e) => {
                            eprintln!("Failed to load Lua plugin {}: {}", path.display(), e);
                        }
                    }
                },
                "wasm" => {
                    match WasmModule::load(&path) {
                        Ok(m) => {
                            endpoint_plugins.insert(endpoint.clone(), PluginInstance::Wasm(Arc::new(m)));
                            loaded_plugins_log.push(format!("{} -> {} [WASM]", endpoint, filename));
                        },
                        Err(e) => {
                            eprintln!("Failed to load WASM plugin {}: {}", path.display(), e);
                        }
                    }
                },
                _ => {
                    eprintln!("Unknown plugin extension for {}: {}", endpoint, filename);
                }
            }
        }
    }
    if !loaded_plugins_log.is_empty() {
        info!("Loaded plugins: {:?}", loaded_plugins_log);
    } else {
        info!("No plugins loaded from mapping");
    }
    let endpoint_plugins = Arc::new(endpoint_plugins);

    // Load Rust dylib plugin at startup
    let mut rust_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    rust_path.push("src/modules/rust_plugin_example/target/release/librust_plugin_example.so");
    let rust_plugin: Arc<Mutex<Option<RustDylibModule>>> = Arc::new(Mutex::new(unsafe {
        if rust_path.exists() {
            match RustDylibModule::load(&rust_path) {
                Ok(m) => Some(m),
                Err(e) => {
                    eprintln!("Failed to load Rust dylib plugin: {}", e);
                    None
                }
            }
        } else {
            eprintln!("Rust dylib plugin not found: {}", rust_path.display());
            None
        }
    }));

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let config = config.clone();
        let chain = chain.clone();
        let endpoint_plugins = endpoint_plugins.clone();
        let rust_plugin_outer = rust_plugin.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
                        let config = config.clone();
                        let chain = chain.clone();
                        let endpoint_plugins = endpoint_plugins.clone();
                        let rust_plugin = rust_plugin_outer.clone();
                        async move {
                            let path = req.uri().path();
                            if let Some(plugin) = endpoint_plugins.get(path) {
                                let input = format!("{} {}", req.method(), req.uri());
                                let resp = match plugin {
                                    PluginInstance::CAbi(p) => {
                                        let output = p.handle(&input);
                                        hyper::Response::new(http_body_util::Full::new(hyper::body::Bytes::from(output)))
                                    },
                                    PluginInstance::Lua(p) => {
                                        let output = p.handle(&input);
                                        hyper::Response::new(http_body_util::Full::new(hyper::body::Bytes::from(output)))
                                    },
                                    PluginInstance::Wasm(p) => {
                                        let output = p.handle(&input);
                                        hyper::Response::new(http_body_util::Full::new(hyper::body::Bytes::from(output)))
                                    },
                                };
                                Ok::<_, std::convert::Infallible>(resp)
                            } else if path == "/reload-rust-plugin" {
                                let mut guard = rust_plugin.lock().unwrap();
                                if let Some(rust_plugin) = guard.as_mut() {
                                    let msg = rust_plugin.reload();
                                    let resp = hyper::Response::new(http_body_util::Full::new(hyper::body::Bytes::from(msg)));
                                    Ok::<_, std::convert::Infallible>(resp)
                                } else {
                                    // Try load if None
                                    let mut rust_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                                    rust_path.push("src/modules/rust_plugin_example/target/release/librust_plugin_example.so");
                                    match unsafe { RustDylibModule::load(&rust_path) } {
                                        Ok(new_mod) => {
                                            *guard = Some(new_mod);
                                            let resp = hyper::Response::new(http_body_util::Full::new(hyper::body::Bytes::from("[rust_plugin] loaded")));
                                            Ok::<_, std::convert::Infallible>(resp)
                                        },
                                        Err(e) => {
                                            let resp = hyper::Response::builder()
                                                .status(500)
                                                .body(http_body_util::Full::new(hyper::body::Bytes::from(format!("[rust_plugin] reload error: {}", e))))
                                                .unwrap();
                                            Ok::<_, std::convert::Infallible>(resp)
                                        }
                                    }
                                }
                            } else if path == "/init-rust-plugin" {
                                let mut guard = rust_plugin.lock().unwrap();
                                if let Some(rust_plugin) = guard.as_mut() {
                                    let msg = rust_plugin.init();
                                    let resp = hyper::Response::new(http_body_util::Full::new(hyper::body::Bytes::from(msg)));
                                    Ok::<_, std::convert::Infallible>(resp)
                                } else {
                                    let resp = hyper::Response::builder()
                                        .status(500)
                                        .body(http_body_util::Full::new(hyper::body::Bytes::from("Rust dylib plugin not loaded")))
                                        .unwrap();
                                    Ok::<_, std::convert::Infallible>(resp)
                                }
                            } else if path == "/shutdown-rust-plugin" {
                                let mut guard = rust_plugin.lock().unwrap();
                                if let Some(rust_plugin) = guard.as_mut() {
                                    let msg = rust_plugin.shutdown();
                                    let resp = hyper::Response::new(http_body_util::Full::new(hyper::body::Bytes::from(msg)));
                                    Ok::<_, std::convert::Infallible>(resp)
                                } else {
                                    let resp = hyper::Response::builder()
                                        .status(500)
                                        .body(http_body_util::Full::new(hyper::body::Bytes::from("Rust dylib plugin not loaded")))
                                        .unwrap();
                                    Ok::<_, std::convert::Infallible>(resp)
                                }
                            } else {
                                // Always read latest config
                                let config_arc = config.clone();
                                chain.handle(req, config_arc).await
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
