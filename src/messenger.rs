use crate::models::{ComponentId, Message};
use std::collections::HashMap;
use std::sync::mpsc::{SendError, Sender};

#[allow(dead_code)]
pub fn get_messenger(
    local_senders: HashMap<ComponentId, Sender<Message>>,
    network_sender: Sender<Message>,
) -> impl FnOnce(Message) -> Result<(), SendError<Message>> {
    move |msg| {
        if let Some(sender) = local_senders.get(&msg.to) {
            sender.send(msg)?;
        } else {
            network_sender.send(msg)?;
        }
        Ok(())
    }
}
