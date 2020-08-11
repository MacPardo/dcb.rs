pub type Timestamp = u64;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Message {
    pub sent_ts: Timestamp,
    pub exec_ts: Timestamp,
    pub from: ComponentId,
    pub to: ComponentId,
    pub id: u32,
    pub content: String,
    pub is_anti: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Checkpoint<State> {
    pub timestamp: Timestamp,
    pub state: State,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ComponentId {
    pub federation_id: u32,
    pub federate_id: u32,
}
