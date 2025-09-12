use crate::middleware_trait::Middleware;
use crate::handler_trait::Handler;
use std::sync::Arc;

pub struct MiddlewareChainBuilder {
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl MiddlewareChainBuilder {
    pub fn new() -> Self {
        MiddlewareChainBuilder {
            middlewares: Vec::new(),
        }
    }

    pub fn add_middleware(mut self, mw: Arc<dyn Middleware>) -> Self {
        self.middlewares.push(mw);
        self
    }

    pub fn build(self, handler: Arc<dyn Handler>) -> Arc<dyn Handler> {
        self.middlewares.into_iter().rfold(handler, |next, mw| {
            Arc::new(MiddlewareHandlerWrapper { mw, next }) as Arc<dyn Handler>
        })
    }
}

struct MiddlewareHandlerWrapper {
    mw: Arc<dyn Middleware>,
    next: Arc<dyn Handler>,
}

impl Handler for MiddlewareHandlerWrapper {
    fn handle<'a>(
        &'a self,
        req: hyper::Request<hyper::body::Incoming>,
        config: std::sync::Arc<crate::config::Config>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<hyper::Response<http_body_util::Full<hyper::body::Bytes>>, std::convert::Infallible>> + Send + 'a>> {
        self.mw.handle(req, config, self.next.clone())
    }
}
