extern crate clap;

use clap::{App, Arg};

use ruft::{ClusterInfo, Node};

fn main() {
    let matches = App::new("ruft")
        .version("0.1")
        .author("JmPotato <ghzpotato@gmail.com>")
        .about("Rust implementation of raft distributed consensus algorithm")
        .arg(
            Arg::with_name("port")
                .long("port")
                .value_name("PORT")
                .help("Sets a cluster start port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("num")
                .long("num")
                .value_name("NUM")
                .help("Sets a cluster node number")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("id")
                .long("id")
                .value_name("ID")
                .help("Sets a node id")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let mut start_port: u16 = 4399;
    let mut node_num: u32 = 5;
    let mut current_node_id = 0;
    let heartbeat_interval: u32 = 5;

    if let Some(port) = matches.value_of("port") {
        println!("Port: {}", port);
        start_port = port.parse::<u16>().unwrap();
    }

    if let Some(num) = matches.value_of("num") {
        println!("Num: {}", num);
        node_num = num.parse::<u32>().unwrap();
    }

    if let Some(id) = matches.value_of("id") {
        println!("ID: {}", id);
        current_node_id = id.parse::<u32>().unwrap();
    }

    let cluster_info = ClusterInfo::new(start_port, node_num, heartbeat_interval);
    let node = Node::new(current_node_id, cluster_info);

    node.run();
}
