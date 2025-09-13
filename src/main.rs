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
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use simple_handler::SimpleHandler;
use std::sync::RwLock;
mod modules {
    pub mod dynamic_loader;
}
use modules::dynamic_loader::{
    CAbiModule, DynamicModule, PluginLifecycle, RustDylibModule, ScriptingModule, WasmModule,
};
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

    // --- PLUGIN LOADERS ---
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

    // Load Lua plugin at startup
    let mut lua_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    lua_path.push("src/modules/lua_plugin_example/hello.lua");
    let lua_plugin: Option<ScriptingModule> = match ScriptingModule::load(&lua_path) {
        Ok(m) => Some(m),
        Err(e) => {
            eprintln!("Failed to load Lua plugin: {}", e);
            None
        }
    };
    let lua_plugin: Arc<Option<ScriptingModule>> = Arc::new(lua_plugin);

    // Load WASM plugin at startup
    let mut wasm_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    wasm_path.push("src/modules/wasm_plugin_example/hello.wasm");
    let wasm_plugin: Option<WasmModule> = match WasmModule::load(&wasm_path) {
        Ok(m) => Some(m),
        Err(e) => {
            eprintln!("Failed to load WASM plugin: {}", e);
            None
        }
    };
    let wasm_plugin: Arc<Option<WasmModule>> = Arc::new(wasm_plugin);

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
        let plugin = plugin.clone();
        let wasm_plugin_outer = wasm_plugin.clone();
        let rust_plugin_outer = rust_plugin.clone();
        let lua_plugin = lua_plugin.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
                        let config = config.clone();
                        let chain = chain.clone();
                        let plugin = plugin.clone();
                        let lua_plugin = lua_plugin.clone();
                        let wasm_plugin = wasm_plugin_outer.clone();
                        let rust_plugin = rust_plugin_outer.clone();
                        async move {
                            // If the request is to /plugin, delegate to the C ABI plugin
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
                            // If the request is to /lua-plugin, delegate to the Lua plugin
                            } else if req.uri().path() == "/lua-plugin" {
                                if let Some(lua_plugin) = lua_plugin.as_ref() {
                                    let input = format!("{} {}", req.method(), req.uri());
                                    let output = lua_plugin.handle(&input);
                                    let resp = hyper::Response::new(http_body_util::Full::new(
                                        hyper::body::Bytes::from(output),
                                    ));
                                    Ok::<_, std::convert::Infallible>(resp)
                                } else {
                                    let resp = hyper::Response::builder()
                                        .status(500)
                                        .body(http_body_util::Full::new(hyper::body::Bytes::from(
                                            "Lua plugin not loaded",
                                        )))
                                        .unwrap();
                                    Ok::<_, std::convert::Infallible>(resp)
                                }
                            // If the request is to /wasm-plugin, delegate to the WASM plugin
                            } else if req.uri().path() == "/wasm-plugin" {
                                if let Some(wasm_plugin) = wasm_plugin.as_ref() {
                                    let input = format!("{} {}", req.method(), req.uri());
                                    let output = wasm_plugin.handle(&input);
                                    let resp = hyper::Response::new(http_body_util::Full::new(
                                        hyper::body::Bytes::from(output),
                                    ));
                                    Ok::<_, std::convert::Infallible>(resp)
                                } else {
                                    let resp = hyper::Response::builder()
                                        .status(500)
                                        .body(http_body_util::Full::new(hyper::body::Bytes::from(
                                            "WASM plugin not loaded",
                                        )))
                                        .unwrap();
                                    Ok::<_, std::convert::Infallible>(resp)
                                }
                            // If the request is to /rust-plugin, delegate to the Rust dylib plugin
                            } else if req.uri().path() == "/rust-plugin" {
                                let guard = rust_plugin.lock().unwrap();
                                if let Some(rust_plugin) = guard.as_ref() {
                                    let input = format!("{} {}", req.method(), req.uri());
                                    let output = rust_plugin.handle(&input);
                                    let resp = hyper::Response::new(http_body_util::Full::new(
                                        hyper::body::Bytes::from(output),
                                    ));
                                    Ok::<_, std::convert::Infallible>(resp)
                                } else {
                                    let resp = hyper::Response::builder()
                                        .status(500)
                                        .body(http_body_util::Full::new(hyper::body::Bytes::from(
                                            "Rust dylib plugin not loaded",
                                        )))
                                        .unwrap();
                                    Ok::<_, std::convert::Infallible>(resp)
                                }
                            } else if req.uri().path() == "/reload-rust-plugin" {
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
                            } else if req.uri().path() == "/init-rust-plugin" {
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
                            } else if req.uri().path() == "/shutdown-rust-plugin" {
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
