use crate::models::MsgCore;

pub trait Component {
    fn init() -> (Self, Vec<MsgCore>)
    where
        Self: Sized;

    fn on_message(self, msg: &MsgCore) -> (Self, Vec<MsgCore>)
    where
        Self: Sized;
}
