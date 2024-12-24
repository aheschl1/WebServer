use std::{self, future::Future};
use core::net::SocketAddr;
use tokio::{self, net::{TcpListener, TcpStream}};

mod shutdown_utils;
mod server_core;
mod router;
mod server_utils;

fn spawn_with_hook(fut: impl Future + Send + 'static, tx: tokio::sync::oneshot::Sender<()>) {
    tokio::spawn(async move {
        fut.await;
        tx.send(()).unwrap();
    });
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error>{
    let config = server_utils::Config::new();
    // connection system
    let endpoint = SocketAddr::from(([127, 0, 0, 1], config.http_port));
    let listener = TcpListener::bind(endpoint).await?;

    // http server shutdown signal
    let (tx_http, rx_http) = tokio::sync::oneshot::channel();
    let http = server_core::start_server(
        listener,
        shutdown_utils::shutdown_on_ctrl_c(),
        10,
        server_core::http::connection_adaptor
    );
    spawn_with_hook(http, tx_http);
    // Now FTP
    let control_endpoint = SocketAddr::from(([127, 0, 0, 1], config.ftp_control_port));
    let control_listener = TcpListener::bind(control_endpoint).await?;

    let (tx_ftp, rx_ftp) = tokio::sync::oneshot::channel();
    let fcp = server_core::start_server(
        control_listener,
        shutdown_utils::shutdown_on_ctrl_c(),
        20,
        server_core::ftp::connection_adaptor
    );
    spawn_with_hook(fcp, tx_ftp);

    // wait for shutdown signal
    rx_http.await.unwrap();
    rx_ftp.await.unwrap();
    Ok(())
}
