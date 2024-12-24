use std::{fs::File, io::Read};

use yaml_rust::YamlLoader;

pub enum FileOpenStatus {
    DNE,
    ERROR,
    SUCCESS
}


pub enum ServerMode {
    HTTP,
    FTP
}

impl PartialEq for ServerMode{
    fn eq(&self, other: &Self) -> bool{
        match (self, other){
            (ServerMode::HTTP, ServerMode::HTTP) => true,
            (ServerMode::FTP, ServerMode::FTP) => true,
            _ => false
        }
    }
}
pub struct Config{
    pub http_port: u16,
    pub ftp_control_port: u16
}

impl Config{
    pub fn new() -> Config{
        // The file is in the crate root, because the src/ directory
        let mut file = File::open("/home/andrewheschl/Documents/WebServer/config.yaml").expect("Could not open config file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Could not read config file");

        let docs = YamlLoader::load_from_str(&contents).unwrap();
        let doc = &docs[0];
        let http_port: u16 = doc["http_port"].as_i64()
            .expect("Could not find http_port") as u16;
        let ftp_control_port : u16 = doc["ftp_control_port"] .as_i64().expect("Cannot find ftp_control_port") as u16;

        Config{
            http_port,
            ftp_control_port
        }
    }
}
