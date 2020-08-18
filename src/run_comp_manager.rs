use crate::messenger::Messenger;
use crate::models::{ComponentCfg, Message};
use std::sync::mpsc::Receiver;

#[allow(dead_code)]
pub fn run_comp_manager(_cfg: ComponentCfg, _messenger: Messenger, _receiver: Receiver<Message>) {}
