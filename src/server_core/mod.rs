
use std::future::Future;
use std::pin::pin;

use async_std::fs::File;
use async_std::io::ReadExt;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Request, Response};
use hyper_util::server::graceful::GracefulShutdown;
use tokio::net::{TcpListener, TcpStream};


use crate::server_utils::FileOpenStatus;

pub mod http;
pub mod ftp;


/**
 * Run a server service using the 'service' method to handle incoming requests.
 * 
 * Serve new connections in the background, until the "shutdown signal" resolves. Graceful shutdown.
 * 
 * # Arguments
 * * `listener` - The TCP listener to accept incoming connections.
 * * `shutdown_signal` - A future that resolves when the server should shutdown.
 * * `shutdown_timeout` - The maximum time to wait for the server to shutdown.
 * * `service` - The service function to handle incoming requests.
 */
pub async fn start_server<T: Future>(
    listener: TcpListener, 
    mut shutdown_signal: T,
    shutdown_timeout: u64,
    connection_adaptor: fn(TcpStream, &GracefulShutdown)
) -> Result<(), std::io::Error>{
    let shutdown_helper = GracefulShutdown::new();
    let mut shutdown_signal = pin!(shutdown_signal);
    loop{
        tokio::select! {
            Ok((stream, _)) = listener.accept() => connection_adaptor(stream, &shutdown_helper),
            _ = &mut shutdown_signal => {
                eprintln!("Starting server shutdown");
                break;
            }
        }
    }
    
    tokio::select! {
        _ = shutdown_helper.shutdown() => {
            eprintln!("Finished shutdown");
        },
        _ = tokio::time::sleep(std::time::Duration::from_secs(shutdown_timeout)) => {
            eprintln!("Shutdown timeout after {shutdown_timeout} seconds. Closing.")
        }
    }
    Ok(())
}

/**
 * Create a BoxBody from a data type which can be converted to bytes.
 * 
 * # Arguments
 * * `body` - The data to be converted to bytes.
 */
pub fn full_box_body<T:Into<Bytes>>(body: T) -> BoxBody<Bytes, hyper::Error>{
    // Build a Full BoxBody from a data type which can be bytes. 
    // Converts Infalliable errors to hyper errors.
    Full::new(body.into())
        .map_err(|n| match n {})
        .boxed()
}

/**
 * Processes a file request, and returns the status, the buffer, and the Content-Type header.
 * 
 * # Arguments
 * * `path` - The path to the file to be processed.
 */
pub async fn process_file_request(path: &str) -> (FileOpenStatus, Option<Vec<u8>>, Option<String>){
    // if the path is a directory, append index.html
    let path = if path.ends_with('/') {
        format!("{}index.html", path)
    } else {
        path.to_string()
    };
    
    let mut file = match File::open(&path).await{
        Ok(file) => file,
        Err(_) => {
            // File does not exist
            return (FileOpenStatus::DNE, None, None);
        }
    };
    let mut buffer = Vec::new();
    if let Err(_) = file.read_to_end(&mut buffer).await{
        return (FileOpenStatus::ERROR, None, None);
    }
    let content_type = match path.to_lowercase().split('.').last(){
        Some("html") => Some("text/html".to_string()),
        Some("css") => Some("text/css".to_string()),
        Some("js") => Some("text/javascript".to_string()),
        Some("json") => Some("application/json".to_string()),
        Some("png") => Some("image/png".to_string()),
        Some("jpg") => Some("image/jpeg".to_string()),
        Some("jpeg") => Some("image/jpeg".to_string()),
        Some("gif") => Some("image/gif".to_string()),
        Some("svg") => Some("image/svg+xml".to_string()),
        Some("ico") => Some("image/x-icon".to_string()),
        Some("pdf") => Some("application/pdf".to_string()),
        _ => Some("text/plain".to_string())
    };
    return (FileOpenStatus::SUCCESS, Some(buffer), content_type);
}