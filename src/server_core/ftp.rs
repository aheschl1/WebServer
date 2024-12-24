use std::pin::Pin;
use std::task::{Context, Poll};
use std::{future::Future, pin::pin};

use hyper_util::server::graceful::{GracefulConnection, GracefulShutdown};
use tokio::net::TcpStream;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

use crate::shutdown_utils::ShutdownHelper;


pub fn connection_adaptor(stream: TcpStream, shutdown_helper: &mut ShutdownHelper){
    let conn = handle_connection(stream);
    let handle = shutdown_helper.register();

    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("Error serving connection: {e}");
        }
        handle.send(()).unwrap();
    });
}

async fn handle_connection(stream: TcpStream) -> Result<(), tokio::io::Error>{
    Ok(())
}

