use std::{self};
use core::net::SocketAddr;
use async_std::path::Path;
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{body::{Body, Bytes}, Method, Request, Response, StatusCode};
use server_core::full_box_body;
use tokio::{self, net::TcpListener};

mod shutdown_utils;
mod server_core;
mod router;

async fn not_implemented(request: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>{
    let mut response = Response::new(request.into_body().boxed());
    *response.status_mut() = StatusCode::NOT_IMPLEMENTED;
    Ok(response)
} 

async fn get_handler(request: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>{
    let request_path = request.uri().path();
    // check that the path exists, else return a 404
    if !(Path::new(request_path).exists().await){
        return Ok(Response::builder()
            .status(404)
            .body(full_box_body(format!("Path {} not found", request_path)))
            .unwrap());
    }
    // read the file and return it as the response body
    Ok(Response::new(request.into_body().boxed()))
}

async fn router(request: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>{
    match request.method() {
        &Method::GET => get_handler(request).await,
        _ => not_implemented(request).await
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error>{
    // connection system
    let endpoint = SocketAddr::from(([127, 0, 0, 1], 6001));
    let listener = TcpListener::bind(endpoint).await?;

    let signal = std::pin::pin!(shutdown_utils::shutdown_on_ctrl_c());
    _ = server_core::start_server(
        listener,
        signal,
        10,
        router
    ).await;

    Ok(())
}
