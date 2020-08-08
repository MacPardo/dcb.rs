pub type Timestamp = u64;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Message {
    // Must not change field order!
    // Because Ord is derived, field order determines how this
    // struct is ordered.
    pub timestamp: Timestamp,
    pub from: ComponentId,
    pub to: ComponentId,
    pub id: u32,
    // end of field order restriction

    // field order can be tinkered with below this comment
    pub content: String,
    pub is_anti: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Checkpoint<AppState> {
    pub timestamp: Timestamp,
    pub state: AppState,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ComponentId {
    pub federation_id: u32,
    pub federate_id: u32,
}
