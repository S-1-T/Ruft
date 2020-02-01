use crate::error::InitializationError;
use super::rpc::{Message, RPCMessage, AppendEntriesRequest, AppendEntriesResponse, RequestVoteRequest, RequestVoteResponse, RPCCS};
use crate::timer::NodeTimer;
use crossbeam_channel::{select, unbounded, Receiver, Sender};
use log::info;
use std::error::Error;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::thread;
use serde::de::Unexpected::Str;

macro_rules! append_entries_request {  //parameter:&self, entries:Vec<String>, peer:String("all")
    ($node:expr, $entries: expr, "all")=>{
        let AERMsg = RPCMessage::new(Message::AppendEntriesRequest(AppendEntriesRequest::new(
            $node.raft_info.current_term,
            $node.raft_info.node_id,
            $node.raft_info.last_applied,
            $node.raft_info.logs.last().unwrap().0,
            $entries,
            $node.raft_info.commit_index,
        )))
        .unwrap();
        $node.rpc.rpc_cs.send_all(&AERMsg).unwrap();
    };
    ($node:expr, $entries: expr, $peer: expr)=>{
        let AERMsg = RPCMessage::new(Message::AppendEntriesRequest(AppendEntriesRequest::new(
            $node.raft_info.current_term,
            $node.raft_info.node_id,
            $node.raft_info.last_applied,
            $node.raft_info.logs.last().unwrap().0,
            $entries,
            $node.raft_info.commit_index,
        )))
        .unwrap();
        $node.rpc.rpc_cs.send_to($peer.to_socket_addrs()?.next().unwrap(),
                                &AERMsg).unwrap();
    };
}

macro_rules! append_entries_response {  //parameter:&self, success:bool, peer:String
    ($node:expr, $success: expr, $peer: expr)=>{
        let AERMsg = RPCMessage::new(Message::AppendEntriesResponse(AppendEntriesResponse::new(
            $node.raft_info.current_term,
            $success,
        )))
        .unwrap();
        $node.rpc.rpc_cs.send_to($peer.to_socket_addrs()?.next().unwrap(),
                                &AERMsg).unwrap();
    };
}

macro_rules! request_vote_request {  //parameter:&self (send to all)
    ($node:expr)=>{
        let RVRMsg = RPCMessage::new(Message::RequestVoteRequest(RequestVoteRequest::new(
            $node.raft_info.current_term,
            $node.raft_info.node_id,
            $node.raft_info.last_applied,
            $node.raft_info.logs.last().unwrap().0,
        )))
        .unwrap();
        $node.rpc.rpc_cs.send_all(&RVRMsg).unwrap();
    };
}

macro_rules! request_vote_response {  //parameter:&self, success:bool
    ($node:expr, $vote_granted: expr, $peer: expr)=>{
        let RVRMsg = RPCMessage::new(Message::RequestVoteResponse(RequestVoteResponse::new(
            $node.raft_info.current_term,
            $vote_granted,
        )))
        .unwrap();
        $node.rpc.rpc_cs.send_to($peer.to_socket_addrs()?.next().unwrap(),
                                &RVRMsg).unwrap();
    };
}

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

struct Rpc {
    rpc_cs: Arc<RPCCS>,
    notifier: Option<Sender<RPCMessage>>,
    receiver: Option<Receiver<RPCMessage>>,
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
        node_list: Vec<String>,   ) -> Result<Node, Box<dyn Error>> {
        if let Some(socket_addr) = format!("{}:{}", host, port).to_socket_addrs()?.next() {
            let mut peer_list: Vec<SocketAddr> = Vec::new();
            for peer in &node_list {
                peer_list.push(peer.as_str().to_socket_addrs()?.next().unwrap());
            }
            let rpc_cs = Arc::new(RPCCS::new(socket_addr, peer_list)?);
            let (rpc_tx, rpc_rx) = unbounded();
            let mut log_vec = Vec::<(u32, String)>::new();  //加入哨兵防止第一次AppendEntriesRequest没有记录
            log_vec.push((0,String::from("Dummy")));
            return Ok(Node {
                cluster_info: ClusterInfo::new(node_number, heartbeat_interval, node_list),
                rpc: Rpc {
                    rpc_cs,
                    notifier: Some(rpc_tx),
                    receiver: Some(rpc_rx),
                },
                raft_info: RaftInfo {
                    node_id,
                    role: Role::Follower,
                    current_term: 0,
                    voted_for: 0,
                    logs: log_vec,
                    commit_index: 0,
                    last_applied: 0,
                    next_index: Vec::<u32>::new(),
                    match_index: Vec::<u32>::new(),
                },
                timer: NodeTimer::new(heartbeat_interval)?,
            });
        }
        Err(Box::new(InitializationError::NodeInitializationError))
    }

    fn change_to(&mut self, new_role: Role) {
        self.raft_info.role = new_role;
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
                    return Err(Box::new(InitializationError::RPCInitializationError));
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
                recv(self.timer.receiver) -> _msg => {
                    // handle the timeout request
                    info!("Timeout occur");
                    if self.raft_info.role.is_candidate() {
                        self.timer.reset_elect();
                    }
                    if self.raft_info.role.is_follower() {
                        self.change_to(Role::Candidate);
                    }
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
