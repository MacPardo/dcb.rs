use crate::models::{ComponentId, Message};
use std::collections::HashMap;
use std::sync::mpsc::{SendError, Sender};

pub struct Messenger {
    pub local_senders: HashMap<ComponentId, Sender<Message>>,
    pub network_sender: Sender<Message>,
}

impl Messenger {
    #[allow(dead_code)]
    pub fn send(&self, msg: Message) -> Result<(), SendError<Message>> {
        if let Some(sender) = self.local_senders.get(&msg.to) {
            sender.send(msg)?;
        } else {
            self.network_sender.send(msg)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn send_local(&self, msg: Message) -> Result<(), SendError<Message>> {
        if let Some(sender) = self.local_senders.get(&msg.to) {
            sender.send(msg)?;
            return Ok(());
        }
        Err(SendError(msg))
    }
}
