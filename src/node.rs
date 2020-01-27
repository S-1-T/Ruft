use crossbeam_channel::{select, unbounded, Receiver, Sender};
use std::net::UdpSocket;
use std::net::{SocketAddr, ToSocketAddrs};
use std::thread;

use crate::rpc::RPCMessage;

struct ClusterInfo {
    node_number: u32,
    majority_number: u32,
    heartbeat_interval: u32,
    node_list: Vec<String>, // Vec(host, port)
}

impl ClusterInfo {
    fn new(node_number: u32, heartbeat_interval: u32, node_list: Vec<String>) -> ClusterInfo {
        let majority_number = (node_number - 1) / 2 + 1;

        ClusterInfo {
            node_number,
            majority_number,
            heartbeat_interval,
            node_list,
        }
    }
}

struct NodeInfo {
    host: String,
    port: u16,
    rpc_notifier: Option<Sender<RPCMessage>>,
    rpc_receiver: Option<Receiver<RPCMessage>>,
    // timer_notifier: Option<Sender<()>>,
    // timer_receiver: Option<Receiver<()>>,
    cluster_info: ClusterInfo,
}

struct RaftInfo {
    node_id: u32,
    current_term: u32,
    voted_for: u32,
    logs: Vec<(u32, String)>,
    commit_index: u32,
    last_applied: u32,
    next_index: Vec<u32>,
    match_index: Vec<u32>,
}

pub struct Node {
    node_info: NodeInfo,
    raft_info: RaftInfo,
}

impl Node {
    pub fn new(
        host: String,
        port: u16,
        node_id: u32,
        node_number: u32,
        heartbeat_interval: u32,
        node_list: Vec<String>,
    ) -> Node {
        let (tx, rx) = unbounded();
        Node {
            node_info: NodeInfo {
                host,
                port,
                rpc_notifier: Some(tx),
                rpc_receiver: Some(rx),
                cluster_info: ClusterInfo::new(node_number, heartbeat_interval, node_list),
            },
            raft_info: RaftInfo {
                node_id,
                current_term: 0,
                voted_for: 0,
                logs: Vec::<(u32, String)>::new(),
                commit_index: 0,
                last_applied: 0,
                next_index: Vec::<u32>::new(),
                match_index: Vec::<u32>::new(),
            },
        }
    }

    fn start_rpc_listener(socket_addr: SocketAddr, rpc_notifier: Sender<RPCMessage>) {
        let socket = UdpSocket::bind(socket_addr).unwrap();
        loop {
            let mut buffer = [0; 1024];
            let (amt, _) = socket.recv_from(&mut buffer).unwrap();
            println!(
                "Receive Raw Data: {}",
                String::from_utf8_lossy(&buffer[..amt])
            );
            if let Ok(msg_content) = String::from_utf8(buffer[..amt].to_vec()) {
                // Handle the raw RPC request from socket buffer
                let msg_parsed = RPCMessage::from_json(msg_content);
                rpc_notifier.send(msg_parsed).unwrap();
            }
        }
    }

    fn start_raft_server(rpc_receiver: Receiver<RPCMessage>) {
        loop {
            select! {
                recv(rpc_receiver) -> msg => {
                    // Handle the RPC request
                    println!("Receive RPC request: {:?}", msg);
                }
            }
        }
    }

    pub fn run(&mut self) {
        if let Some(tx) = self.node_info.rpc_notifier.take() {
            if let Some(socket_addr) = format!("{}:{}", self.node_info.host, self.node_info.port)
                .to_socket_addrs()
                .unwrap()
                .next()
            {
                println!(
                    "Starting RPC Server on {}:{}",
                    self.node_info.host, self.node_info.port
                );
                thread::spawn(move || {
                    Node::start_rpc_listener(socket_addr, tx);
                });
            }
        }
        if let Some(tr) = self.node_info.rpc_receiver.take() {
            println!("Starting Raft Algorithm");
            Node::start_raft_server(tr);
        }
    }
}
