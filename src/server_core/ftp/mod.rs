mod status;
mod utils;

use std::borrow::Cow;
use std::fmt::format;
use std::pin::Pin;
use std::task::{Context, Poll};

use async_std::fs::File;
use async_std::io::ReadExt;
use async_std::stream;
use tokio::net::TcpStream;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

use crate::shutdown_utils::ShutdownHelper;
use status::{ConnectionState, TransferType, TransferMode, TransferStructure};
use utils::hash_password;

/**
 * Handle a new connection.
 * 
 * This function is called when a new connection is made to the server, and is the starting point for handling the connection.
 * 
 * Spawns handle_connection as a tokio task, and registers a shutdown handle.
 */
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
    let mut transfer_type = TransferType::Ascii;
    let mut transfer_mode = TransferMode::Stream;
    let mut transfer_structure = TransferStructure::File;

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
            "USER" => { // Login with username
                auth_state = do_login_flow(command.split(' ').nth(1).unwrap_or("annonymous"), &mut stream)
                    .await
                    .unwrap_or(ConnectionState::NotLoggedIn);
                match auth_state{
                    ConnectionState::LoggedIn => Some("230 User logged in".to_string()),
                    ConnectionState::Annonymous => Some("230 User logged in".to_string()),
                    _ => Some("530 Log in unsuccessful".to_string()),
                }
            },
            "QUIT" => { // Disconnect
                auth_state = ConnectionState::Disconnected;
                Some("221 Goodbye".to_string())
            },
            "PORT" => { // Setup active transfer mode
                if let Ok(stream_result) = make_active_mode_data_connection(&input).await{
                    data_stream = stream_result;
                }else{
                    data_stream = None;
                }
                match &data_stream{
                    Some(_) => Some("200 PORT command successful".to_string()),
                    None => Some("425 Can't open data connection.".to_string())
                }
            },
            "TYPE" => { // Set transfer type
                transfer_type = TransferType::from(input.split(' ').nth(1).unwrap_or("A"));
                let reply = format!("200 Type set to {}", transfer_type);
                Some(reply)
            },
            "MODE" => {
                transfer_mode = TransferMode::from(input.split(' ').nth(1).unwrap_or("S"));
                let reply = format!("200 Transfer mode set to {}", transfer_mode);
                Some(reply)
            },
            "STRU" => {
                transfer_structure = TransferStructure::from(input.split(' ').nth(1).unwrap_or("F"));
                let reply = format!("200 Transfer structure set to {}", transfer_structure);
                Some(reply)
            },
            "RETR" => {
                let path = input.split(' ').nth(1); 
                match (path, data_stream.as_mut()){
                    (_, None) => Some("425 No data connection established.".to_string()),
                    (Some(path), Some(mut ds)) => {
                        let result = retrieve_file(
                            path, 
                            &mut ds,
                            transfer_mode.clone(), 
                            transfer_type.clone(), 
                            transfer_structure.clone(), 
                            auth_state.clone()
                        ).await;
                        match result{
                            Ok(_) => Some("226 Transfer complete.".to_string()),
                            Err(_) => Some("451 Requested action aborted.".to_string())
                        }
                    },
                    (None, _) => Some("501 No file name given.".to_string())
                }
            }
            _ => Some("502 This service not implemented.".to_string())
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

async fn retrieve_file(
    path: &str, 
    stream: &mut TcpStream, 
    mode: TransferMode, 
    data_type: TransferType, 
    structure: TransferStructure, 
    auth_state: ConnectionState) -> Result<String, tokio::io::Error>
{
    // make sure structure is File, or send error NOT IMPLEMENTED
    if structure != TransferStructure::File{
        return Ok(String::from("504 Command not implemented for that parameter. (Can only handle File STRU)"));
    }

    if !utils::auth_can_access_file(path, auth_state){
        return Ok(String::from("550 Permission denied."));
    }
    // we need to make sure the file actually exists.
    let file = match File::open(path).await{
        Ok(file) => file,
        Err(_) => return Ok(String::from("550 File not found."))
    };
    // based on the transfer mode, we need to send the file in the correct way.
    match mode {
        TransferMode::Stream => {
            let mut reader = file;
            let mut buffer = [0u8; 1024];
            loop{
                let bytes_read = reader.read(&mut buffer).await?;
                if bytes_read == 0{
                    break;
                }
                match data_type {
                    TransferType::Ascii => {
                        let mut ascii = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
                        ascii = ascii.replace("\n", "\r\n");
                        stream.write_all(ascii.as_bytes()).await?;
                    },
                    TransferType::Binary => {
                        stream.write_all(&buffer[..bytes_read]).await?;
                    },
                    _ => {
                        // Error, we don't support this type.
                        return Err(io::Error::new(io::ErrorKind::Other, "Unsupported data type."));
                    }
                }
            }
        },
        _ => {
            // Server error
            return Err(io::Error::new(io::ErrorKind::Other, "Server error."));
        }
    }

    Ok(String::from("226 Transfer complete."))
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