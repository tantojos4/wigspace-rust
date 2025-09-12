













use crate::config::Config;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};
use log::info;
use std::convert::Infallible;
use std::sync::Arc;

pub async fn handle_request<B>(
    req: Request<B>,
    _config: Arc<Config>,
) -> Result<Response<Full<Bytes>>, Infallible>
where
    B: Send + 'static,
{
    // Dummy: get remote_addr from header (for real, use hyper::server::conn::AddrStream)
    let remote_addr = req
        .headers()
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-");
    let method = req.method();
    let uri = req.uri();
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-");
    let status = 200; // Static for now, update if dynamic response
    // Nginx-style: $remote_addr - - [time] "METHOD URI" status "user-agent"
    info!(
        "{} - - \"{} {}\" {} \"{}\"",
        remote_addr, method, uri, status, user_agent
    );
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}

