use hyper::{Method, Uri};
use wigspace_rust::simple_handler::simple_handler_response;

#[test]
fn test_handle_request_hello_world() {
    let method = Method::GET;
    let uri: Uri = "/test".parse().unwrap();
    let body_str = simple_handler_response(&method, &uri);
    assert!(body_str.contains("SimpleHandler"));
}
