use crate::models::Message;
use std::io::prelude::*;
use std::net::{TcpListener, ToSocketAddrs};
use std::sync::mpsc::Sender;

#[allow(dead_code)]
pub fn run_listener<Addr: ToSocketAddrs>(address: Addr, sender: Sender<Message>) {
    let listener = TcpListener::bind(address).unwrap();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        let msg = String::from_utf8(buffer.to_vec()).unwrap();
        let msg = msg.replace('\n', "");
        let msg = msg.trim_matches(char::from(0));
        let msg = serde_json::from_str(&msg.to_owned()).unwrap();
        sender.send(msg).unwrap();
    }
}
