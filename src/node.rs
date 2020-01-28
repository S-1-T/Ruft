use crossbeam_channel::{select, unbounded, Receiver, Sender};
use rand::Rng;
use std::error::Error;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::thread::{self, sleep};
use std::time::Duration;

use crate::error::InitializationError;
use crate::rpc::{Message, RPCMessage, RPCCS};

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
    rpc_cs: Arc<RPCCS>,
    rpc_notifier: Option<Sender<RPCMessage>>,
    rpc_receiver: Option<Receiver<RPCMessage>>,
    timeout_notifier: Option<Sender<()>>,
    timeout_receiver: Option<Receiver<()>>,
}

// Role of a Node
#[derive(PartialEq, Copy, Clone)]
enum Role {
    Follower,
    Candidate,
    Leader,
}

impl Role {
    fn is_follower(self) -> bool {
        self == Role::Follower
    }

    fn is_candidate(self) -> bool {
        self == Role::Candidate
    }

    fn is_leader(self) -> bool {
        self == Role::Leader
    }
}

struct RaftInfo {
    node_id: u32,
    role: Role,
    current_term: u32,
    voted_for: u32,
    logs: Vec<(u32, String)>,
    commit_index: u32,
    last_applied: u32,
    next_index: Vec<u32>,
    match_index: Vec<u32>,
}

pub struct Node {
    cluster_info: ClusterInfo,
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
    ) -> Result<Node, Box<dyn Error>> {
        let (rpc_tx, rpc_rx) = unbounded();
        let (timeout_tx, timeout_rx) = unbounded();
        if let Some(socket_addr) = format!("{}:{}", host, port).to_socket_addrs()?.next() {
            let mut peer_list: Vec<SocketAddr> = Vec::new();
            for peer in &node_list {
                peer_list.push(peer.as_str().to_socket_addrs()?.next().unwrap());
            }
            let rpc_cs = Arc::new(RPCCS::new(socket_addr, peer_list)?);
            return Ok(Node {
                cluster_info: ClusterInfo::new(node_number, heartbeat_interval, node_list),
                node_info: NodeInfo {
                    rpc_cs,
                    rpc_notifier: Some(rpc_tx),
                    rpc_receiver: Some(rpc_rx),
                    timeout_notifier: Some(timeout_tx),
                    timeout_receiver: Some(timeout_rx),
                },
                raft_info: RaftInfo {
                    node_id,
                    role: Role::Follower,
                    current_term: 0,
                    voted_for: 0,
                    logs: Vec::<(u32, String)>::new(),
                    commit_index: 0,
                    last_applied: 0,
                    next_index: Vec::<u32>::new(),
                    match_index: Vec::<u32>::new(),
                },
            });
        }
        Err(Box::new(InitializationError::NodeInitializationError))
    }

    fn change_to(&mut self, new_role: Role) {
        self.raft_info.role = new_role;
    }

    fn start_rpc_listener(&mut self) -> Result<(), Box<dyn Error>> {
        println!(
            "Starting RPC Server/Client on {}",
            self.node_info.rpc_cs.socket_addr
        );
        if let Some(rpc_notifier) = self.node_info.rpc_notifier.take() {
            let rpc_cs = Arc::clone(&self.node_info.rpc_cs);
            thread::spawn(move || match rpc_cs.start_listener(rpc_notifier) {
                Ok(()) => Ok(()),
                Err(error) => {
                    println!("RPC Clent/Server start_listener error: {}", error);
                    return Err(Box::new(InitializationError::RPCInitializationError));
                }
            });
        };
        Ok(())
    }

    fn start_timer(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Starting timer");
        if let Some(timeout_tx) = self.node_info.timeout_notifier.take() {
            thread::spawn(move || {
                loop {
                    let interval: u64 = rand::thread_rng().gen_range(300, 500);
                    sleep(Duration::from_millis(interval));
                    // generate the request: ()
                    timeout_tx.send(()).unwrap();
                    println!("timeout after {} milliseconds", interval);
                }
            });
        }
        Ok(())
    }

    fn start_raft_server(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Starting Raft Algorithm");
        if let Some(rpc_receiver) = self.node_info.rpc_receiver.take() {
            if let Some(timeout_receiver) = self.node_info.timeout_receiver.take() {
                loop {
                    select! {
                        recv(rpc_receiver) -> msg => {
                            // Handle the RPC request
                            let msg = msg?;
                            println!("Receive RPC request: {:?}", msg.message);
                            match msg.message {
                                Message::AppendEntriesRequest(request) => {
                                    // To-do: Handle AppendEntries
                                },
                                Message::AppendEntriesResponse(request) => {
                                    // To-do: Handle AppendEntries
                                },
                                Message::RequestVoteRequest(request) => {
                                    // To-do: Handle RequestVote
                                },
                                Message::RequestVoteResponse(request) => {
                                    // To-do: Handle RequestVote
                                },
                            }
                        }
                        recv(timeout_receiver) -> _msg => {
                            // handle the timeout request
                            println!("Timeout occur");
                            if self.raft_info.role.is_candidate() {
                                // Election failed
                            }
                            if !self.raft_info.role.is_leader() {
                                self.raft_info.role = Role::Candidate;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // RPC Server/Client Thread
        self.start_rpc_listener()?;

        // Timer Thread
        self.start_timer()?;

        // Main Thread
        self.start_raft_server()?;

        Ok(())
    }
}
