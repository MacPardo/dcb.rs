use crate::component::Component;
use crate::gateway::Gateway;
use crate::models::{ComponentId, Message, MsgCore, Timestamp};
use std::collections::HashMap;

#[allow(dead_code)]
pub struct Translator {
    pub local_id: ComponentId,
    pub route_to_dest: HashMap<String, (ComponentId, String)>,
}

impl Translator {
    pub fn translate(&self, msg_core: MsgCore, sent_ts: Timestamp) -> Message {
        let (destination_id, destination_route) = &self.route_to_dest[&msg_core.route];
        Message {
            id: 0,
            is_anti: false,
            from: self.local_id,
            to: destination_id.clone(),
            sent_ts: sent_ts,
            exec_ts: msg_core.exec_ts,
            route: destination_route.clone(),
            payload: msg_core.payload,
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

    fn on_message(&self, state: State, message: Message) -> (State, Vec<Message>) {
        let message = MsgCore::new(message);
        let (new_state, messages) = state.on_message(message.exec_ts, &message);
        let messages = messages
            .into_iter()
            .map(|m| self.translate(m, message.exec_ts))
            .collect();
        (new_state, messages)
    }
}
