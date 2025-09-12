use hyper::{Request, Response};
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::convert::Infallible;
use crate::config::Config;
use crate::handler_trait::Handler;

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
        config: Arc<Config>,
        next: Arc<dyn Handler + Send + Sync>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Full<Bytes>>, Infallible>> + Send + 'a>> {
        let next = next.clone();
        let method = req.method().clone();
        let uri = req.uri().clone();
        Box::pin(async move {
            println!("[LOG] {} {}", method, uri);
            next.handle(req, config).await
        })
    }
}
