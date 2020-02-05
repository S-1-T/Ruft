#[macro_use]
pub mod macros;

use crate::entry::Entry;

use crossbeam_channel::{Receiver, Sender};
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::Arc;

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub enum Message {
    AppendEntriesRequest(AppendEntriesRequest),
    AppendEntriesResponse(AppendEntriesResponse),
    RequestVoteRequest(RequestVoteRequest),
    RequestVoteResponse(RequestVoteResponse),
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct AppendEntriesRequest {
    pub term: u32,
    pub leader_addr: SocketAddr,
    pub prev_log_index: usize,
    pub prev_log_term: u32,
    pub entries: Vec<Entry>,
    pub leader_commit: usize,
}

impl AppendEntriesRequest {
    pub fn new(
        term: u32,
        leader_addr: SocketAddr,
        prev_log_index: usize,
        prev_log_term: u32,
        entries: Vec<Entry>,
        leader_commit: usize,
    ) -> AppendEntriesRequest {
        AppendEntriesRequest {
            term,
            leader_addr,
            prev_log_index,
            prev_log_term,
            entries,
            leader_commit,
        }
    }
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct AppendEntriesResponse {
    pub socket_addr: SocketAddr,
    pub next_index: usize,
    pub match_index: usize,
    pub term: u32,
    pub success: bool,
}

impl AppendEntriesResponse {
    pub fn new(
        socket_addr: SocketAddr,
        next_index: usize,
        match_index: usize,
        term: u32,
        success: bool,
    ) -> AppendEntriesResponse {
        AppendEntriesResponse {
            socket_addr,
            next_index,
            match_index,
            term,
            success,
        }
    }
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct RequestVoteRequest {
    pub term: u32,
    pub candidated_addr: SocketAddr,
    pub last_log_index: usize,
    pub last_log_term: u32,
}

impl RequestVoteRequest {
    pub fn new(
        term: u32,
        candidated_addr: SocketAddr,
        last_log_index: usize,
        last_log_term: u32,
    ) -> RequestVoteRequest {
        RequestVoteRequest {
            term,
            candidated_addr,
            last_log_index,
            last_log_term,
        }
    }
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct RequestVoteResponse {
    pub term: u32,
    pub vote_granted: bool,
}

impl RequestVoteResponse {
    pub fn new(term: u32, vote_granted: bool) -> RequestVoteResponse {
        RequestVoteResponse { term, vote_granted }
    }
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct RPCMessage {
    pub message: Message,
}

impl RPCMessage {
    pub fn new(message: Message) -> Result<RPCMessage, Box<dyn Error>> {
        Ok(RPCMessage { message })
    }

    pub fn from_json(json_str: String) -> Result<RPCMessage, Box<dyn Error>> {
        let rpc_message: RPCMessage = serde_json::from_str(json_str.as_str())?;

        Ok(rpc_message)
    }
}

pub struct RPCCS {
    socket: UdpSocket,
    pub socket_addr: SocketAddr,
    peer_list: Vec<SocketAddr>,
}

// RPC Client & Server
impl RPCCS {
    pub fn new(
        socket_addr: SocketAddr,
        peer_list: Vec<SocketAddr>,
    ) -> Result<RPCCS, Box<dyn Error>> {
        let socket = UdpSocket::bind(socket_addr)?;
        Ok(RPCCS {
            socket,
            socket_addr,
            peer_list,
        })
    }

    pub fn start_listener(&self, rpc_notifier: Sender<RPCMessage>) -> Result<(), Box<dyn Error>> {
        loop {
            let mut buffer = [0; 1024];
            let (amt, _) = match self.socket.recv_from(&mut buffer) {
                Ok(pair) => pair,
                Err(_) => {
                    error!("{} receive error", self.socket_addr.port());
                    (0, "127.0.0.1:8006".to_socket_addrs()?.next().unwrap())
                }
            };
            if let Ok(msg_content) = String::from_utf8(buffer[..amt].to_vec()) {
                // Handle the raw RPC request from socket buffer
                let msg_parsed = RPCMessage::from_json(msg_content)?;
                rpc_notifier.send(msg_parsed)?;
            }
        }
    }

    // Send request to one node in peer_list
    pub fn send_to(
        &self,
        recv_node: SocketAddr,
        message_to_send: &RPCMessage,
    ) -> Result<(), Box<dyn Error>> {
        //recv_node: host, port
        let msg_parsed = json!(message_to_send).to_string();
        let buffer = msg_parsed.as_bytes();
        self.socket.send_to(&buffer, recv_node)?;
        Ok(())
    }

    // Send request to all nodes in peer_list, eg, heartbeat
    pub fn send_all(&self, message_to_send: &RPCMessage) -> Result<(), Box<dyn Error>> {
        for peer in &self.peer_list {
            let msg_parsed = json!(message_to_send).to_string();
            let buffer = msg_parsed.as_bytes();
            self.socket.send_to(&buffer, peer)?;
        }
        Ok(())
    }
}

pub struct Rpc {
    pub cs: Arc<RPCCS>,
    pub notifier: Option<Sender<RPCMessage>>,
    pub receiver: Option<Receiver<RPCMessage>>,
}
