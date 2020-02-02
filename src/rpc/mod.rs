#[macro_use]
pub mod macros;

use crossbeam_channel::{Sender, Receiver};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;
use std::net::{SocketAddr, UdpSocket};
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
    pub leader_id: u32,
    pub prev_log_index: u32,
    pub prev_log_term: u32,
    pub entries: Vec<String>,
    pub leader_commit: u32,
}

impl AppendEntriesRequest {
    pub fn new(
        term: u32,
        leader_id: u32,
        prev_log_index: u32,
        prev_log_term: u32,
        entries: Vec<String>,
        leader_commit: u32,
    ) -> AppendEntriesRequest {
        AppendEntriesRequest {
            term,
            leader_id,
            prev_log_index,
            prev_log_term,
            entries,
            leader_commit,
        }
    }
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct AppendEntriesResponse {
    pub term: u32,
    pub success: bool,
}

impl AppendEntriesResponse {
    pub fn new(term: u32, success: bool) -> AppendEntriesResponse {
        AppendEntriesResponse { term, success }
    }
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct RequestVoteRequest {
    pub term: u32,
    pub candidated_id: u32,
    pub last_log_index: u32,
    pub last_log_term: u32,
}

impl RequestVoteRequest {
    pub fn new(
        term: u32,
        candidated_id: u32,
        last_log_index: u32,
        last_log_term: u32,
    ) -> RequestVoteRequest {
        RequestVoteRequest {
            term,
            candidated_id,
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
            let (amt, _) = self.socket.recv_from(&mut buffer)?;
            info!(
                "Receive Raw Data: {}",
                String::from_utf8_lossy(&buffer[..amt])
            );
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
    pub rpc_cs: Arc<RPCCS>,
    pub notifier: Option<Sender<RPCMessage>>,
    pub receiver: Option<Receiver<RPCMessage>>,
}