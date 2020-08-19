use crate::models::{MsgCore, Timestamp};

pub trait Component {
    fn init() -> (Self, Vec<MsgCore>)
    where
        Self: Sized;

    fn on_message(self, lvt: Timestamp, msg: &MsgCore) -> (Self, Vec<MsgCore>)
    where
        Self: Sized;
}
