use crate::error::InitializationError;
use crate::timer::NodeTimer;
use crate::rpc::*;
use crate::raft::*;

use crossbeam_channel::{select, unbounded};
use log::info;
use serde::de::Unexpected::Str;
use std::error::Error;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
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
    cluster_info: ClusterInfo,
    raft_info: RaftInfo,
    rpc: Rpc,
    timer: NodeTimer,
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
        if let Some(socket_addr) = format!("{}:{}", host, port).to_socket_addrs()?.next() {
            let mut peer_list: Vec<SocketAddr> = Vec::new();
            for peer in &node_list {
                peer_list.push(peer.as_str().to_socket_addrs()?.next().unwrap());
            }
            let rpc_cs = Arc::new(RPCCS::new(socket_addr, peer_list)?);
            let (rpc_tx, rpc_rx) = unbounded();
            let mut log_vec = Vec::<(u32, String)>::new(); //加入哨兵防止第一次AppendEntriesRequest没有记录
            log_vec.push((0, String::from("Dummy")));
            return Ok(Node {
                cluster_info: ClusterInfo::new(node_number, heartbeat_interval, node_list),
                rpc: Rpc {
                    rpc_cs,
                    notifier: Some(rpc_tx),
                    receiver: Some(rpc_rx),
                },
                raft_info: RaftInfo {
                    node_id,
                    role: Follower::new().unwrap(),
                    current_term: 0,
                    voted_for: 0,
                    logs: log_vec,
                    commit_index: 0,
                    last_applied: 0,
                },
                timer: NodeTimer::new(heartbeat_interval)?,
            });
        }
        Err(Box::new(InitializationError::NodeInitializationError))
    }

    fn change_to(&mut self, rolename: Role) {
        match rolename {
            Role::Follower => self.raft_info.role = Follower::new().unwrap(),
            Role::Candidate => self.raft_info.role = Candidate::new().unwrap(),
            Role::Leader => self.raft_info.role = Leader::new().unwrap(),
        }
    }

    fn start_rpc_listener(&mut self) -> Result<(), Box<dyn Error>> {
        info!(
            "Starting RPC Server/Client on {}",
            self.rpc.rpc_cs.socket_addr
        );
        if let Some(rpc_notifier) = self.rpc.notifier.take() {
            let rpc_cs = Arc::clone(&self.rpc.rpc_cs);
            thread::spawn(move || match rpc_cs.start_listener(rpc_notifier) {
                Ok(()) => Ok(()),
                Err(error) => {
                    info!("RPC Clent/Server start_listener error: {}", error);
                    Err(Box::new(InitializationError::RPCInitializationError))
                }
            });
        };
        Ok(())
    }

    fn start_timer(&mut self) -> Result<(), Box<dyn Error>> {
        info!("Starting Timer");
        self.timer.run_elect();
        Ok(())
    }

    fn start_raft_server(&mut self) -> Result<(), Box<dyn Error>> {
        info!("Starting Raft Algorithm");
        loop {
            select! {
                recv(self.rpc.receiver.as_ref().unwrap()) -> msg => {
                    // Handle the RPC request
                    let msg = msg?;
                    info!("Receive RPC request: {:?}", msg.message);
                    match msg.message {
                        Message::AppendEntriesRequest(request) => {
                            info!("Message: Append entries request");
                            self.raft_info.role.handle_append_entries_request();
                        },
                        Message::AppendEntriesResponse(request) => {
                            info!("Message: Append entries response");
                            self.raft_info.role.handle_append_entries_response();
                        },
                        Message::RequestVoteRequest(request) => {
                            info!("Message: Request vote request");
                            self.raft_info.role.handle_request_vote_request();
                        },
                        Message::RequestVoteResponse(request) => {
                            info!("Message: Request vote response");
                            self.raft_info.role.handle_request_vote_response();
                        },
                    }
                }
                recv(self.timer.receiver) -> msg => {
                    // handle the timeout request
                    info!("Timeout occur");
                    self.raft_info.role.handle_timeout();
                }
            }
        }
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
