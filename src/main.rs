mod dependency_vector;
mod message_queue;
mod models;
mod network;
mod rollback_manager;

use std::collections::BinaryHeap;

fn main() {
    let mut h = BinaryHeap::new();
    h.push(2);
    h.push(2);

    println!("{:?}", h.pop());
    println!("{:?}", h.pop());
}
