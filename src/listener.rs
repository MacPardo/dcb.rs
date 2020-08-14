use std::io::prelude::*;
use std::net::{TcpListener, ToSocketAddrs};
use std::sync::mpsc::Sender;

#[allow(dead_code)]
pub fn run_listener<Addr: ToSocketAddrs>(address: Addr, sender: Sender<String>) {
    let listener = TcpListener::bind(address).unwrap();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        let message = String::from_utf8(buffer.to_vec()).unwrap();
        sender.send(message).unwrap();
    }
}
