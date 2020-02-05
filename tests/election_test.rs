extern crate simple_logger;

use log::info;
use ruft::Node;
use std::thread;

macro_rules! start_node {
    ($id: expr) => {
        let mut peers = vec![
            String::from("127.0.0.1:8000"),
            String::from("127.0.0.1:8001"),
            String::from("127.0.0.1:8002"),
            String::from("127.0.0.1:8003"),
            String::from("127.0.0.1:8004"),
        ];
        peers.remove($id);
        let node = Node::new(String::from("127.0.0.1"), 8000 + ($id as u16), 5, 50, peers);
        let mut node = match node {
            Ok(node) => node,
            Err(error) => panic!("Creating Node Error: {}", error),
        };
        match node.run() {
            Ok(()) => info!("Node Stopped"),
            Err(error) => panic!("Running Node Error: {}", error),
        };
    };
}

#[test]
fn elect_five_nodes() {
    simple_logger::init().unwrap();

    for id in 0..5 {
        thread::spawn(move || {
            start_node!(id);
        });
    }
    thread::sleep_ms(3000);
}
