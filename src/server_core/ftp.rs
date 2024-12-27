use std::borrow::Cow;
use std::fmt::format;
use std::pin::Pin;
use std::task::{Context, Poll};

use async_std::fs::File;
use async_std::io::ReadExt;
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
    
    let mut auth_state = ConnectionState::NotLoggedIn;
    let mut data_stream: Option<TcpStream> = None;

    loop{
        let bytes_read = stream.read(&mut buffer).await?;
        if bytes_read == 0{
            break;
        }

        let input = String::from_utf8_lossy(&buffer[..bytes_read]);
        let input = input.trim();
        println!("{input}");
        
        let command = input.split(' ').next().unwrap_or("Bye");
        // let response = get_response(command, &input, &mut auth_state, &mut stream).await?;

        let response = match command{
            "USER" => {
                auth_state = do_login_flow(command.split(' ').nth(1).unwrap_or("annonymous"), &mut stream)
                    .await
                    .unwrap_or(ConnectionState::NotLoggedIn);
                match auth_state{
                    ConnectionState::LoggedIn => Some("230 User logged in"),
                    ConnectionState::Annonymous => Some("230 User logged in"),
                    _ => Some("530 Log in unsuccessful"),
                }
            },
            "QUIT" => {
                auth_state = ConnectionState::Disconnected;
                Some("221 Goodbye")
            },
            "PORT" => {
                if let Ok(stream_result) = make_active_mode_data_connection(&input).await{
                    data_stream = stream_result;
                };
                match data_stream{
                    Some(_) => Some("200 PORT command successful"),
                    None => Some("425 Can't open data connection.")
                }
            },
            _ => Some("502 This service not implemented.")
        };
        if let Some(response) = response{
            stream.write_all(response.as_bytes()).await?;
        }

        if auth_state == ConnectionState::Disconnected{
            break;
        }
    }

    Ok(())
}

async fn make_active_mode_data_connection(input: &str) -> Result<Option<TcpStream>, tokio::io::Error>{
    let mut parts = input.split(' ').skip(1); // remove command
    let mut parts = parts.next().unwrap().split(',');            // split the ip and port

    let ip = format!(
        "{first}.{second}.{third}.{fourth}",
        first = parts.next().unwrap(),
        second = parts.next().unwrap(),
        third = parts.next().unwrap(),
        fourth = parts.next().unwrap()
    );
    let port = format!(
        "{}{}",
        parts.next().unwrap(),
        parts.next().unwrap()
    );

    let addr = format!("{ip}:{port}");
    let stream = tokio::select!{
        Ok(stream) = TcpStream::connect(addr) => Some(stream),
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => None
    };
    Ok(stream)
}

async fn do_login_flow(username: &str, stream: &mut TcpStream) -> Result<ConnectionState, std::io::Error>{
    // we have an original username, now, we try to authenticate them. return Ok(()) when done.
    if username == "anonymous"{
        return Ok(ConnectionState::Annonymous);
    }
    let mut password_ok = false;
    let mut attempts = 0;
    while !password_ok && attempts < 5{
        attempts += 1;
        stream.write_all("331 Password required for {username}.".as_bytes()).await?;
        let mut buffer = [0u8; 1024];
        let bytes_read = stream.read(&mut buffer).await?;
        let input = String::from_utf8_lossy(&buffer[..bytes_read]);
        let command = input.split(' ').next().unwrap_or("Bye");
        if command == "PASS"{
            let password = input.split(' ').nth(1).unwrap_or("");
            password_ok = check_password(username, password).await;
            if !password_ok{
                stream.write_all(format!("530 Login incorrect {} attempts remaining.", 5-attempts).as_bytes()).await?;
            }
        }else{
            return Ok(ConnectionState::NotLoggedIn);
        }
    }

    Ok(ConnectionState::LoggedIn)
}

async fn check_password(username: &str, password: &str) -> bool{
    // read the hashed password from ~/.ftp_server/username.passwd
    // compare the hashed password with the password given.
    let password_hash: String = hash_password(password);
    let actual_hash = match File::open(format!("/home/andrewheschl/ftp_server/{username}.passwd")).await{
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents).await.unwrap();
            Some(contents)
        },
        Err(_) => None 
    };
    if let Some(hash) = actual_hash  {
        return password_hash == hash;
    }
    return false;
}

/**
 * Hash the password using sha256.
 */
fn hash_password(password: &str) -> String{
    // hash the password using a secure hashing algorithm.
    // return the hashed password.
    sha256::digest(password)
}

enum ConnectionState{
    NotLoggedIn,
    Disconnected,
    LoggedIn,
    Annonymous
}

impl PartialEq for ConnectionState{
    fn eq(&self, other: &Self) -> bool{
        match (self, other){
            (ConnectionState::NotLoggedIn, ConnectionState::NotLoggedIn) => true,
            (ConnectionState::Disconnected, ConnectionState::Disconnected) => true,
            (ConnectionState::LoggedIn, ConnectionState::LoggedIn) => true,
            (ConnectionState::Annonymous, ConnectionState::Annonymous) => true,
            _ => false
        }
    }
}
