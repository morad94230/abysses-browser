use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

pub async fn start_proxy(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });
    let server = Server::bind(&addr).serve(make_svc);
    server.await?;
    Ok(())
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let host = req
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    if host.ends_with(".abyss") {
        let name = host.trim_end_matches(".abyss");
        let html = format!(
            "<html><head><title>{} - Abysses</title><style>body{{background:#0a0a1a;color:#00d4ff;font-family:monospace;padding:40px;}}</style></head><body><h1>{}</h1><p>Hosted on Abysses decentralized darkweb.</p></body></html>",
            name, name
        );
        Ok(Response::builder()
            .status(200)
            .header("content-type", "text/html")
            .body(Body::from(html))
            .unwrap())
    } else if req.uri().path() == "/health" {
        Ok(Response::builder()
            .status(200)
            .body(Body::from("OK"))
            .unwrap())
    } else {
        Ok(Response::builder()
            .status(403)
            .body(Body::from("Abysses only serves .abyss domains"))
            .unwrap())
    }
}
