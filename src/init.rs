use crate::messenger::Messenger;
use crate::models::{ComponentId, Message};
use crate::network;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

#[allow(dead_code)]
pub fn init(addr: String, remote_addrs: HashMap<ComponentId, String>) {
    let (net_sender, net_receiver) = mpsc::channel::<Message>();

    let messenger = Messenger {
        local_senders: HashMap::new(),
        network_sender: net_sender,
    };
    let server_handle = thread::spawn(move || network::run_server(addr, &messenger));
    let client_handle = thread::spawn(move || network::run_client(&remote_addrs, net_receiver));

    server_handle.join().unwrap();
    client_handle.join().unwrap();
}
