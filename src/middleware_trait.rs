use hyper::{Request, Response};
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use crate::config::Config;
use std::convert::Infallible;

pub trait Middleware: Send + Sync {
    fn handle<'a>(
        &'a self,
        req: Request<Incoming>,
        config: Arc<Config>,
        next: Arc<dyn Handler + Send + Sync>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Full<Bytes>>, Infallible>> + Send + 'a>>;
}

// Import Handler trait for the next parameter
use crate::handler_trait::Handler;
