use crate::models::Message;

pub trait Gateway<State> {
    fn init(&self) -> (State, Vec<Message>);

    fn on_message(&self, state: State, message: &Message) -> (State, Vec<Message>);
}
