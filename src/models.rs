use serde::{Deserialize, Serialize};

pub type Timestamp = u64;
pub type ComponentId = u16;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Message {
    pub sent_ts: Timestamp,
    pub exec_ts: Timestamp,
    pub from: ComponentId,
    pub to: ComponentId,
    pub payload: String,
    pub path: String,
    pub id: u32,
    pub is_anti: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct MsgCore {
    pub payload: String,
    pub path: String,
    pub exec_ts: Timestamp,
}

impl MsgCore {
    pub fn new(m: Message) -> MsgCore {
        MsgCore {
            payload: m.payload,
            path: m.path,
            exec_ts: m.exec_ts,
        }
    }
}

impl Message {
    #[allow(dead_code)]
    pub fn is_inverse_of(&self, other: &Self) -> bool {
        self.sent_ts == other.sent_ts
            && self.exec_ts == other.exec_ts
            && self.from == other.from
            && self.to == other.to
            && self.id == other.id
            && self.is_anti != other.is_anti
    }

    #[allow(dead_code)]
    pub fn get_anti(&self) -> Result<Message, ()> {
        if self.is_anti {
            return Err(());
        }
        let mut msg = self.clone();
        msg.is_anti = true;
        Ok(msg)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Checkpoint<State> {
    pub timestamp: Timestamp,
    pub state: State,
}

#[derive(Clone, Copy)]
pub struct ComponentCfg {
    pub id: ComponentId,
}
