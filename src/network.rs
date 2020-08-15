use crate::messenger::Messenger;
use crate::models::{ComponentId, Message};
use std::collections::HashMap;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::mpsc::Receiver;

const BUFFER_SIZE: usize = 1024;

#[allow(dead_code)]
pub fn run_server(address: impl ToSocketAddrs, messenger: &Messenger) {
    let listener = TcpListener::bind(address).unwrap();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut buffer = [0u8; BUFFER_SIZE];
        stream.read(&mut buffer).unwrap();
        let msg = String::from_utf8(buffer.to_vec()).unwrap();
        let msg = msg.trim_matches(char::from(0));
        let msg = msg.replace('\n', "");
        let msg = serde_json::from_str(&msg.to_owned()).unwrap();
        messenger.send_local(msg).unwrap();
    }
}

#[allow(dead_code)]
pub fn run_client(
    addresses: &HashMap<ComponentId, impl ToSocketAddrs>,
    receiver: Receiver<Message>,
) {
    for msg in receiver {
        let addr = addresses.get(&msg.to).unwrap();
        let msg = serde_json::to_string(&msg).unwrap();
        let mut stream = TcpStream::connect(addr).unwrap();
        stream.write(msg.as_bytes()).unwrap();
    }
}
