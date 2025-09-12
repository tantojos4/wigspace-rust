pub fn simple_handler_response(method: &hyper::Method, uri: &hyper::Uri) -> String {
    format!("SimpleHandler: {} {}", method, uri)
}
use crate::handler_trait::Handler;
use crate::config::Config;
use hyper::{Request, Response, StatusCode};
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::convert::Infallible;
use tokio::fs;
use tokio::io::AsyncReadExt;

pub struct SimpleHandler;

impl Handler for SimpleHandler {
    fn handle<'a>(
        &'a self,
        req: Request<Incoming>,
        config: Arc<Config>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Full<Bytes>>, Infallible>> + Send + 'a>> {
        Box::pin(async move {
            // Serve static files if static_dir is set
            if let Some(ref static_dir) = config.static_dir {
                let mut path = req.uri().path().trim_start_matches('/').to_string();
                if path.is_empty() { path = "index.html".to_string(); }
                let file_path = std::path::Path::new(static_dir).join(&path);
                if let Ok(mut file) = fs::File::open(&file_path).await {
                    let mut buf = Vec::new();
                    if let Ok(_) = file.read_to_end(&mut buf).await {
                        return Ok(Response::new(Full::new(Bytes::from(buf))));
                    }
                }
            }
            // Fallback: echo method and URI
            let body = simple_handler_response(req.method(), req.uri());
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from(body)))
                .unwrap())
        })
    }
}
