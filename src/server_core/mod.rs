use std::future::Future;

use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use hyper_util::server::graceful::GracefulShutdown;
use tokio;
use tokio::net::TcpListener;
use hyper::server::conn::http1;

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
pub async fn start_server<T, F, Fut>(
    listener: TcpListener, 
    mut shutdown_signal: T,
    shutdown_timeout: u64,
    service: F) -> Result<(), std::io::Error>
where 
    T: Future + Unpin,
    F: Copy + Fn(Request<hyper::body::Incoming>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>> + Send + 'static, // The `Future` returned by `service`
{
    let shutdown_helper = GracefulShutdown::new();

    loop{
        tokio::select! {
            Ok((stream, _)) = listener.accept() => {
                let io = TokioIo::new(stream);
                let conn = http1::Builder::new().serve_connection(io, service_fn(service));
                let fut = shutdown_helper.watch(conn);
                tokio::spawn(async {
                    if let Err(e) = fut.await{
                        eprintln!("Error serving connection: {e}");
                    }
                });

            },
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