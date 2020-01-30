use crossbeam_channel::Sender;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::net::{SocketAddr, UdpSocket};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    AppendEntriesRequest(AppendEntriesRequest),
    AppendEntriesResponse(AppendEntriesResponse),
    RequestVoteRequest(RequestVoteRequest),
    RequestVoteResponse(RequestVoteResponse),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppendEntriesRequest {
    term: u32,
    leader_id: u32,
    prev_log_index: u32,
    prev_log_term: u32,
    entries: Vec<String>,
    leader_commit: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppendEntriesResponse {
    term: u32,
    success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestVoteRequest {
    term: u32,
    candidated_id: u32,
    last_log_index: u32,
    last_log_term: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestVoteResponse {
    term: u32,
    vote_granted: bool,
}
#[derive(Serialize, Deserialize, Debug)]
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
            println!(
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
    pub fn send_to(&self, recv_node: SocketAddr, message_to_send: RPCMessage) -> Result<(), Box<dyn Error>> {    //recv_node: host, port
        let msg_parsed = json!(message_to_send).to_string();
        let buffer = msg_parsed.as_bytes();
        self.socket.send_to(&buffer, recv_node)?;
        Ok(())
    }

    // Send request to all nodes in peer_list, eg, heartbeat
    pub fn send_all(&self, message_to_send: RPCMessage) -> Result<(), Box<dyn Error>> {
        for peer in &self.peer_list {
            let msg_parsed = json!(message_to_send).to_string();
            let buffer = msg_parsed.as_bytes();
            self.socket.send_to(&buffer, peer)?;
        }
        Ok(())
    }
}
