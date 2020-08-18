use crate::messenger::Messenger;
use crate::models::{ComponentCfg, ComponentId, Message};
use crate::network::{run_client, run_server};
use crate::run_comp_manager::run_comp_manager;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

#[allow(dead_code)]
pub fn init(
    addr: String,
    remote_addrs: HashMap<ComponentId, String>,
    local_components: Vec<ComponentCfg>,
) {
    let (net_sender, net_receiver) = channel::<Message>();

    let local_components: Vec<(ComponentCfg, Sender<Message>, Receiver<Message>)> =
        local_components
            .into_iter()
            .map(|c| {
                let (s, r) = channel();
                (c, s, r)
            })
            .collect();

    let mut local_senders = HashMap::new();
    local_components.iter().for_each(|tuple| {
        local_senders.insert(tuple.0.id, tuple.1.clone());
    });

    let messenger = Messenger {
        local_senders: local_senders.clone(),
        network_sender: net_sender,
    };

    let messenger_clone = messenger.clone();
    let server_handle = thread::spawn(move || run_server(addr, messenger_clone));
    let client_handle = thread::spawn(move || run_client(&remote_addrs, net_receiver));

    // let mut handles = Vec::new();
    // for tuple in local_components {
    //     let messenger_clone = messenger.clone();
    // let handle = thread::spawn(move || run_comp_manager(tuple.0, messenger_clone, tuple.2));
    // handles.push(handle);
    // }

    // for handle in handles {
    // handle.join().unwrap();
    // }
    server_handle.join().unwrap();
    client_handle.join().unwrap();
}
