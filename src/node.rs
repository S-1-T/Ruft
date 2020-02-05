use crate::entry::Entry;
use crate::error::InitializationError;
use crate::rpc::*;
use crate::timer::NodeTimer;

use crossbeam_channel::{select, unbounded};
use log::{error, info};
use std::collections::HashMap;
use std::error::Error;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::thread;

#[macro_use]
use crate::*;

struct ClusterInfo {
    node_number: u32,
    majority_number: u32,
    heartbeat_interval: u32,
    node_list: Vec<String>, // Vec("host:port")
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

// Role of a Node
#[derive(PartialEq, Copy, Clone)]
enum Role {
    Follower,
    Candidate,
    Leader,
}

pub struct Node {
    cluster_info: ClusterInfo,
    role: Role,
    current_term: u32,
    candidated_addr: Option<SocketAddr>,
    leader_addr: Option<SocketAddr>,
    votes: u32,
    logs: Vec<Entry>,
    commit_index: usize,
    last_applied: u32,
    next_index: HashMap<SocketAddr, usize>,
    match_index: HashMap<SocketAddr, usize>,
    pub rpc: Rpc,
    timer: NodeTimer,
}

impl Node {
    pub fn new(
        host: String,
        port: u16,
        node_number: u32,
        heartbeat_interval: u32,
        node_list: Vec<String>,
    ) -> Result<Node, Box<dyn Error>> {
        if let Some(socket_addr) = format!("{}:{}", host, port).to_socket_addrs()?.next() {
            let mut peer_list: Vec<SocketAddr> = Vec::new();
            let mut next_index: HashMap<SocketAddr, usize> = HashMap::new();
            let mut match_index: HashMap<SocketAddr, usize> = HashMap::new();
            for peer in &node_list {
                let peer = peer.as_str().to_socket_addrs()?.next().unwrap();
                peer_list.push(peer.clone());
                next_index.insert(peer.clone(), 0);
                match_index.insert(peer.clone(), 0);
            }
            let cs = Arc::new(RPCCS::new(socket_addr, peer_list)?);
            let (rpc_tx, rpc_rx) = unbounded();
            let mut log_vec = Vec::<Entry>::new(); //加入哨兵防止第一次AppendEntriesRequest没有记录
            log_vec.push(Entry {
                index: 1,
                term: 0,
                command: String::new(),
            });
            return Ok(Node {
                cluster_info: ClusterInfo::new(node_number, heartbeat_interval, node_list),
                role: Role::Follower,
                current_term: 0,
                candidated_addr: None,
                leader_addr: None,
                votes: 0,
                logs: log_vec,
                commit_index: 1,
                last_applied: 1,
                next_index,
                match_index,
                rpc: Rpc {
                    cs,
                    notifier: Some(rpc_tx),
                    receiver: Some(rpc_rx),
                },
                timer: NodeTimer::new(heartbeat_interval)?,
            });
        }
        Err(Box::new(InitializationError::NodeInitializationError))
    }

    fn start_rpc_listener(&mut self) -> Result<(), Box<dyn Error>> {
        info!("Starting RPC Server/Client on {}", self.rpc.cs.socket_addr);
        if let Some(rpc_notifier) = self.rpc.notifier.take() {
            let rpc_cs = Arc::clone(&self.rpc.cs);
            thread::spawn(move || match rpc_cs.start_listener(rpc_notifier) {
                Ok(()) => Ok(()),
                Err(error) => {
                    error!(
                        "{} RPC Clent/Server error: {}",
                        rpc_cs.socket_addr.port(),
                        error
                    );
                    Err(Box::new(InitializationError::RPCInitializationError))
                }
            });
        };
        Ok(())
    }

    fn start_raft_server(&mut self) -> Result<(), Box<dyn Error>> {
        info!("Starting Raft Algorithm");
        self.timer.run_elect();
        loop {
            if self.role == Role::Leader {
                info!("leader {} closed", self.rpc.cs.socket_addr.port());
                break;
            }
            select! {
                recv(self.rpc.receiver.as_ref().unwrap()) -> msg => {
                    // Handle the RPC request
                    let msg = msg?;
                    info!(
                        "{} receive RPC request: {:?}",
                        self.rpc.cs.socket_addr.port(), msg.message
                    );
                    match msg.message {
                        Message::AppendEntriesRequest(request) => {
                            self.handle_append_entries_request(request);
                        },
                        Message::AppendEntriesResponse(request) => {
                            self.handle_append_entries_response(request);
                        },
                        Message::RequestVoteRequest(request) => {
                            self.handle_request_vote_request(request);
                        },
                        Message::RequestVoteResponse(request) => {
                            self.handle_request_vote_response(request);
                        },
                    }
                }
                recv(self.timer.receiver) -> _ => {
                    info!("{} timeout occur", self.rpc.cs.socket_addr.port());
                    self.handle_timeout();
                }
            }
        }
        Ok(())
    }

    fn handle_append_entries_request(&mut self, mut msg: AppendEntriesRequest) {
        match self.role {
            Role::Follower => {
                self.timer.reset_elect();

                if !msg.entries.is_empty() {
                    let success: bool = if msg.term < self.current_term
                        || msg.prev_log_index >= self.logs.len()
                        || msg.entries[msg.prev_log_index].term
                            != self.logs[msg.prev_log_index].term
                    {
                        false
                    } else {
                        self.logs.pop();
                        self.logs.append(&mut msg.entries);
                        if msg.leader_commit > self.commit_index {
                            self.commit_index = if msg.leader_commit < self.logs.len() {
                                msg.leader_commit
                            } else {
                                self.logs.len()
                            };
                            true
                        } else {
                            false
                        }
                    };
                    append_entries_response!(&self, success, msg.leader_addr);
                }
                self.leader_addr = Some(msg.leader_addr);
                self.candidated_addr = None;
            }
            Role::Candidate => {
                if msg.term >= self.current_term {
                    self.current_term = msg.term;
                    self.change_role_to(Role::Follower);
                    self.timer.run_elect();
                    // let follower handle it
                    // self.handle_append_entries_request(msg);
                }
            }
            Role::Leader => {
                if self.current_term <= msg.term {
                    self.timer.stop_heartbeat();
                    self.change_role_to(Role::Follower);
                    self.timer.run_elect();
                }
            }
        }
    }

    fn handle_append_entries_response(&mut self, msg: AppendEntriesResponse) {
        match self.role {
            Role::Follower => {}
            Role::Candidate => {}
            Role::Leader => {
                if msg.success {
                    *self.next_index.entry(msg.socket_addr).or_insert(0) = msg.next_index;
                    *self.match_index.entry(msg.socket_addr).or_insert(0) = msg.match_index;

                    let mut i = self.commit_index;
                    loop {
                        let mut match_count = 1;
                        for val in self.match_index.values() {
                            if val >= &i && self.logs[i].term == self.current_term {
                                match_count += 1;
                            }
                        }
                        if match_count >= self.cluster_info.majority_number {
                            i += 1;
                        } else {
                            break;
                        }
                    }
                    self.commit_index = i;
                }
            }
        }
    }

    fn handle_request_vote_request(&mut self, msg: RequestVoteRequest) {
        match self.role {
            Role::Follower => {
                self.timer.reset_elect();
                if msg.term >= self.current_term
                    && (self.candidated_addr.is_none()
                        || self.candidated_addr.unwrap() == msg.candidated_addr)
                    && msg.last_log_term >= self.logs.last().unwrap().term
                    && msg.last_log_index >= self.logs.last().unwrap().index
                {
                    self.current_term = msg.term;
                    vote_for!(&self, true, msg.candidated_addr);
                    self.candidated_addr = Some(msg.candidated_addr);
                } else {
                    vote_for!(&self, false, msg.candidated_addr);
                }
            }
            Role::Candidate => {
                if msg.term > self.current_term {
                    self.current_term = msg.term;
                    self.change_role_to(Role::Follower);
                    self.timer.run_elect();
                }
                vote_for!(&self, true, msg.candidated_addr);
            }
            Role::Leader => {}
        }
    }

    fn handle_request_vote_response(&mut self, msg: RequestVoteResponse) {
        match self.role {
            Role::Follower => {}
            Role::Candidate => {
                if msg.term > self.current_term {
                    self.current_term = msg.term;
                    self.change_role_to(Role::Follower);
                    self.timer.run_elect();
                } else if msg.vote_granted {
                    self.votes += 1;
                    info!(
                        "{} gets {} votes",
                        self.rpc.cs.socket_addr.port(),
                        self.votes
                    );
                    if self.votes >= self.cluster_info.majority_number {
                        self.change_role_to(Role::Leader);
                        info!(
                            "{} is leader in term {}",
                            self.rpc.cs.socket_addr.port(),
                            self.current_term
                        );
                        self.timer.run_heartbeat();
                        append_entries_request!(&self, Vec::<Entry>::new()); // heartbeat
                    }
                }
            }
            Role::Leader => {}
        }
    }

    fn handle_timeout(&mut self) {
        match self.role {
            Role::Follower => {
                self.change_role_to(Role::Candidate);
                self.timer.reset_elect();
                self.current_term += 1;
                info!(
                    "{} is candidate in term {}",
                    self.rpc.cs.socket_addr.port(),
                    self.current_term
                );
                self.candidated_addr = Some(self.rpc.cs.socket_addr);
                self.votes = 1;
                request_vote!(&self);
            }
            Role::Candidate => {
                self.timer.run_elect();
            }
            Role::Leader => {
                append_entries_request!(&self, Vec::<Entry>::new()); // heartbeat
            }
        }
    }

    fn change_role_to(&mut self, rolename: Role) {
        self.role = rolename;
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // RPC Server/Client Thread
        self.start_rpc_listener()?;

        // Main Thread
        self.start_raft_server()?;

        Ok(())
    }
}
