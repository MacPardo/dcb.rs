use crate::gateway::Gateway;
use crate::messenger::Messenger;
use crate::models::ComponentId;
use crate::msg_queue::MsgQueue;
use crate::rollback_manager::RollbackManager;
use std::sync::Arc;

#[allow(dead_code)]
pub fn consume_msg_queue<State: Clone>(
    component_id: ComponentId,
    gateway: impl Gateway<State>,
    should_take_checkpoint: fn(&State, &RollbackManager<State>) -> bool,
    messenger: Arc<Messenger>,
    queue: Arc<MsgQueue>,
) {
    let (initial_state, initial_messages) = gateway.init();
    for msg in initial_messages {
        messenger.send(msg).unwrap();
    }

    let mut rollback_manager = RollbackManager::new(component_id, initial_state.clone());
    let mut current_state = initial_state;

    loop {
        let received = queue.pop();

        let violates_lcc = received.exec_ts < rollback_manager.lvt();
        if violates_lcc {
            let msgs = rollback_manager.rollback(received.exec_ts).unwrap();
            for msg in msgs {
                messenger.send(msg).unwrap();
            }
        }

        if received.exec_ts > rollback_manager.lvt()
            && should_take_checkpoint(&current_state, &rollback_manager)
        {
            rollback_manager.take_checkpoint();
        }

        rollback_manager.save_message(received.clone()).unwrap();

        let ts = received.exec_ts;
        let (new_state, msgs) = gateway.on_message(current_state, received);
        rollback_manager.update(new_state.clone(), ts).unwrap();
        current_state = new_state;

        for msg in msgs {
            rollback_manager.save_message(msg.clone()).unwrap();
            messenger.send(msg).unwrap();
        }
    }
}
