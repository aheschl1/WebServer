use std::{self};
use core::net::SocketAddr;
use async_std::{fs::File, io::ReadExt, path::Path};
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{body::{Body, Bytes}, header::HeaderName, Method, Request, Response, StatusCode};
use server_core::{full_box_body, FileOpenStatus};
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
    // read the file and return it as the response body

    let (status, buffer, content_type) = server_core::process_file_request(request_path).await; 
    match status{
        FileOpenStatus::DNE => {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full_box_body(format!("File {} not found", request_path)))
                .unwrap();
            Ok(response)
        },
        FileOpenStatus::ERROR => {
            let response = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(full_box_body("Internal server error"))
                .unwrap();
            Ok(response)
        },
        FileOpenStatus::SUCCESS => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", content_type.unwrap())
                .body(full_box_body(buffer.unwrap()))
                .unwrap();
            Ok(response)
        }
    }

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
