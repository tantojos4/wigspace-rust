use crate::config::Config;
use crate::handler_trait::Handler;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response};
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::RwLock;

pub struct LoggingMiddleware;

impl LoggingMiddleware {
    pub fn new() -> Self {
        LoggingMiddleware
    }
}

impl super::middleware_trait::Middleware for LoggingMiddleware {
    fn handle<'a>(
        &'a self,
        req: Request<Incoming>,
        config: Arc<RwLock<Config>>,
        next: Arc<dyn Handler + Send + Sync>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Full<Bytes>>, Infallible>> + Send + 'a>> {
        let next = next.clone();
        let method = req.method().clone();
        let uri = req.uri().clone();
        Box::pin(async move {
            let response = next.handle(req, config).await;
            let status = response.as_ref().map(|r| r.status().as_u16()).unwrap_or(0);
            log::info!("[access] {} {} -> {}", method, uri, status);
            response
        })
    }
}
