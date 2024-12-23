use std::{self};
use core::net::SocketAddr;
use tokio::{self, net::TcpListener};

mod shutdown_utils;
mod server_core;
mod router;
mod server_utils;


#[tokio::main]
async fn main() -> Result<(), std::io::Error>{
    let config = server_utils::Config::new();
    // connection system
    let endpoint = SocketAddr::from(([127, 0, 0, 1], config.http_port));
    let listener = TcpListener::bind(endpoint).await?;

    let signal = std::pin::pin!(shutdown_utils::shutdown_on_ctrl_c());
    _ = server_core::start_server(
        listener,
        signal,
        10,
        server_core::http::router
    ).await;

    Ok(())
}
