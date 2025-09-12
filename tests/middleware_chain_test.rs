use hyper::{Method, Uri};
use std::sync::Arc;
use wigspace_rust::config::Config;
use wigspace_rust::handler_trait::Handler;
use wigspace_rust::logging_middleware::LoggingMiddleware;
use wigspace_rust::middleware_chain::MiddlewareChainBuilder;
use wigspace_rust::simple_handler::SimpleHandler;
use wigspace_rust::simple_handler::simple_handler_response;

#[tokio::test]
async fn test_logging_middleware_chain() {
    let _config = Arc::new(Config::default());
    let handler: Arc<dyn Handler> = Arc::new(SimpleHandler);
    let logging_middleware = Arc::new(LoggingMiddleware::new());
    let _config = Arc::new(Config::default());
    let _chain = MiddlewareChainBuilder::new()
        .add_middleware(logging_middleware)
        .build(handler);

    let method = Method::GET;
    let uri: Uri = "/test".parse().unwrap();
    let body_str = simple_handler_response(&method, &uri);
    assert!(body_str.contains("SimpleHandler"));
}
