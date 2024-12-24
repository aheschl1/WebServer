use std::pin::Pin;
use std::task::{Context, Poll};

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

async fn handle_connection(mut stream: TcpStream) -> Result<(), tokio::io::Error>{
    let mut buffer = [0u8; 1024];
    stream.write_all("220 Welcome to ftp server :()".as_bytes()).await?;
    
    loop{
        let bytes_read = stream.read(&mut buffer).await?;
        if bytes_read == 0{
            break;
        }

        let input = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("{input}");
        
        let response = match input.trim() {
            "USER anonymous" => "331 User name okay, need password\r\n",
            "PASS" => "230 User logged in, proceed\r\n",
            "QUIT" => {
                stream.write_all(b"221 Goodbye\r\n").await?;
                break;
            }
            "LIST" => "150 Here comes the directory listing\r\n",
            _ => "502 Command not implemented\r\n",
        };

        stream.write_all(response.as_bytes()).await?;

    }

    Ok(())
}

