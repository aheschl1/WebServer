use async_std::{fs::File, io::ReadExt, path::Path};
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{body::{Body, Bytes}, header::HeaderName, Method, Request, Response, StatusCode};

use crate::server_core::{self, full_box_body};
use crate::server_utils;

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
        server_utils::FileOpenStatus::DNE => {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full_box_body(format!("File {} not found", request_path)))
                .unwrap();
            Ok(response)
        },
        server_utils::FileOpenStatus::ERROR => {
            let response = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(full_box_body("Internal server error"))
                .unwrap();
            Ok(response)
        },
        server_utils::FileOpenStatus::SUCCESS => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", content_type.unwrap())
                .body(full_box_body(buffer.unwrap()))
                .unwrap();
            Ok(response)
        }
    }

}

pub async fn router(request: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>{
    match request.method() {
        &Method::GET => get_handler(request).await,
        _ => not_implemented(request).await
    }
}