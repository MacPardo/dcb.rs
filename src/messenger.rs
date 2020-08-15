use crate::models::{ComponentId, Message};
use std::collections::HashMap;
use std::sync::mpsc::Sender;

#[allow(dead_code)]
pub fn get_messenger(
    local_senders: HashMap<ComponentId, Sender<Message>>,
    network_sender: Sender<Message>,
) -> impl FnOnce(Message) {
    move |msg| {
        if let Some(sender) = local_senders.get(&msg.to) {
            sender.send(msg).unwrap();
        } else {
            network_sender.send(msg).unwrap();
        }
    }
}
