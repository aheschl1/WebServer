use std::{self};
use core::net::SocketAddr;
use http_body_util::combinators::BoxBody;
use hyper::{body::Bytes, Request, Response};
use tokio::{self, net::TcpListener};

mod shutdown_utils;
mod server_core;
mod router;

async fn service_router(request: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>{
    Ok(Response::new(server_core::full_box_body("Hello World!")))
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error>{
    // connection system
    let endpoint = SocketAddr::from(([127, 0, 0, 1], 6001));
    let listener = TcpListener::bind(endpoint).await?;

    let routed_service = router::routed_service(
        service_router,
        Some(service_router),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None
    );
    let signal = std::pin::pin!(shutdown_utils::shutdown_on_ctrl_c());
    _ = server_core::start_server(
        listener,
        signal,
        10,
        routed_service
    ).await;

    Ok(())
}
