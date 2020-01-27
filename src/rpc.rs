use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
enum Message {
    AppendEntriesRequest(AppendEntriesRequest),
    AppendEntriesResponse(AppendEntriesResponse),
    RequestVoteRequest(RequestVoteRequest),
    RequestVoteResponse(RequestVoteResponse),
}

#[derive(Serialize, Deserialize, Debug)]
struct AppendEntriesRequest {
    term: u32,
    leader_id: u32,
    prev_log_index: u32,
    prev_log_term: u32,
    entries: Vec<String>,
    leader_commit: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct AppendEntriesResponse {
    term: u32,
    success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestVoteRequest {
    term: u32,
    candidated_id: u32,
    last_log_index: u32,
    last_log_term: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestVoteResponse {
    term: u32,
    vote_granted: bool,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct RPCMessage {
    message: Message,
}

impl RPCMessage {
    pub fn from_json(json_str: String) -> RPCMessage {
        let rpc_message: RPCMessage = serde_json::from_str(json_str.as_str()).unwrap();

        rpc_message
    }
}
