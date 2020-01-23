use crossbeam_channel::{unbounded, Receiver, Sender};
use std::net::UdpSocket;
use std::net::{SocketAddr, ToSocketAddrs};
use std::thread;

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

pub struct Node {
    host: String,
    port: u16,
    rpc_notifier: Option<Sender<()>>,
    rpc_receiver: Option<Receiver<()>>,
    cluster_info: ClusterInfo,
}

impl Node {
    pub fn new(
        host: String,
        port: u16,
        node_number: u32,
        heartbeat_interval: u32,
        node_list: Vec<String>,
    ) -> Node {
        let (tx, rx) = unbounded();
        Node {
            host,
            port,
            rpc_notifier: Some(tx),
            rpc_receiver: Some(rx),
            cluster_info: ClusterInfo::new(node_number, heartbeat_interval, node_list),
        }
    }

    fn start_rpc_listener(socket_addr: SocketAddr, rpc_notifier: Sender<()>) {
        let socket = UdpSocket::bind(socket_addr).unwrap();
        loop {
            let mut buffer = [0; 1024];
            let (amt, _) = socket.recv_from(&mut buffer).unwrap();
            print!(
                "Receive Raw Data: {}",
                String::from_utf8_lossy(&buffer[..amt])
            );
            if let Ok(msg_content) = String::from_utf8(buffer[..amt].to_vec()) {
                // Handle the raw RPC request from socket buffer
                rpc_notifier.send(()).unwrap();
            }
        }
    }

    fn start_raft_server(rpc_receiver: Receiver<()>) {
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
        if let Some(tx) = self.rpc_notifier.take() {
            if let Some(socket_addr) = format!("{}:{}", self.host, self.port)
                .to_socket_addrs()
                .unwrap()
                .next()
            {
                println!("Starting RPC Server on {}:{}", self.host, self.port);
                thread::spawn(move || {
                    Node::start_rpc_listener(socket_addr, tx);
                });
            }
        }
        if let Some(tr) = self.rpc_receiver.take() {
            println!("Starting Raft Algorithm");
            Node::start_raft_server(tr);
        }
    }
}
