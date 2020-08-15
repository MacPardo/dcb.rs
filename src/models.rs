use serde::{Deserialize, Serialize};

pub type Timestamp = u64;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Message {
    pub sent_ts: Timestamp,
    pub exec_ts: Timestamp,
    pub from: ComponentId,
    pub to: ComponentId,
    pub id: u32,
    pub content: String,
    pub is_anti: bool,
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
        msg.content = String::new();
        msg.is_anti = true;
        Ok(msg)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Checkpoint<State> {
    pub timestamp: Timestamp,
    pub state: State,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ComponentId {
    pub federation_id: u32,
    pub federate_id: u32,
}
