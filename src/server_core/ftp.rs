use std::{future::Future, pin::pin};

use hyper_util::server::graceful::GracefulShutdown;
use tokio::net::{TcpListener, TcpStream};


pub fn connection_adaptor(stream: TcpStream, shutdown_helper: &GracefulShutdown){

}