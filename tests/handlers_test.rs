use wigspace_rust::handlers::handle_request;
use wigspace_rust::config::Config;
use hyper::{Request, Method};
use http_body_util::{BodyExt, Empty};
use hyper::body::Bytes;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[test]
fn test_handle_request_hello_world() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let config = Arc::new(Config {
            address: "127.0.0.1".to_string(),
            port: 8080,
            access_log: None,
            error_log: None,
            static_dir: None,
            proxy_pass: None,
        });

        let req = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .header("user-agent", "UnitTest")
            .body(Empty::<Bytes>::new())
            .unwrap();
        let resp = handle_request(req, config).await.unwrap();
        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body_bytes, "Hello, World!");
    });
}
