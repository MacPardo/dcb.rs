use crate::component::Component;
use crate::gateway::Gateway;
use crate::models::{ComponentId, Message, MsgContent, Timestamp};
use std::collections::HashMap;

#[allow(dead_code)]
pub struct Translator {
    pub local_id: ComponentId,
    pub path_to_id: HashMap<String, ComponentId>,
}

impl Translator {
    pub fn translate(&self, msg_content: MsgContent, sent_ts: Timestamp) -> Message {
        let destination = self.path_to_id[&msg_content.path];
        Message {
            content: msg_content,
            id: 0,
            is_anti: false,
            from: self.local_id,
            to: destination,
            sent_ts: sent_ts,
        }
    }
}

impl<State> Gateway<State> for Translator
where
    State: Component,
{
    fn init(&self) -> (State, Vec<Message>) {
        let (initial_state, messages) = State::init();
        let messages = messages.into_iter().map(|m| self.translate(m, 0)).collect();
        (initial_state, messages)
    }

    fn on_message(&self, state: State, message: &Message) -> (State, Vec<Message>) {
        let (new_state, messages) = state.on_message(message.content.exec_ts, &message.content);
        let messages = messages
            .into_iter()
            .map(|m| self.translate(m, message.content.exec_ts))
            .collect();
        (new_state, messages)
    }
}
