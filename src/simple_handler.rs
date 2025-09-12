pub fn simple_handler_response(method: &hyper::Method, uri: &hyper::Uri) -> String {
    format!("SimpleHandler: {} {}", method, uri)
}
use crate::handler_trait::Handler;
use crate::config::Config;
use hyper::{Request, Response};
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::convert::Infallible;

pub struct SimpleHandler;

impl Handler for SimpleHandler {
    fn handle<'a>(
        &'a self,
        req: Request<Incoming>,
        _config: Arc<Config>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Full<Bytes>>, Infallible>> + Send + 'a>> {
        Box::pin(async move {
            let body = simple_handler_response(req.method(), req.uri());
            Ok(Response::new(Full::new(Bytes::from(body))))
        })
    }
}
