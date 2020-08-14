mod dependency_vector;
mod listener;
mod message_queue;
mod models;
mod rollback_manager;

use listener::run_listener;
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || run_listener("127.0.0.1:8888", tx));

    for msg in rx {
        println!("got msg <{}>", msg);
    }
}
