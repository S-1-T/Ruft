extern crate clap;

use clap::{App, Arg};

use ruft::Node;

fn main() {
    let matches = App::new("ruft")
        .version("0.1")
        .author("JmPotato <ghzpotato@gmail.com>")
        .about("Rust implementation of raft distributed consensus algorithm")
        .arg(
            Arg::with_name("host")
                .long("host")
                .value_name("HOST")
                .help("Sets the node's host")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .value_name("PORT")
                .help("Sets the node's port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("num")
                .long("num")
                .value_name("NUM")
                .help("Sets the cluster's node number")
                .takes_value(true),
        )
        .get_matches();

    let mut node_host: String = String::from("127.0.0.1");
    let mut node_port: u16 = 5299;
    let mut node_num: u32 = 5;
    let heartbeat_interval: u32 = 5;
    let mut node_list: Vec<String> = Vec::new();

    if let Some(host) = matches.value_of("host") {
        println!("Host: {}", host);
        node_host = String::from(host);
    }

    if let Some(port) = matches.value_of("port") {
        println!("Port: {}", port);
        node_port = port.parse::<u16>().unwrap();
    }

    if let Some(num) = matches.value_of("num") {
        println!("Num: {}", num);
        node_num = num.parse::<u32>().unwrap();
    }

    let mut node = Node::new(
        node_host,
        node_port,
        node_num,
        heartbeat_interval,
        node_list,
    );

    node.run();
}
