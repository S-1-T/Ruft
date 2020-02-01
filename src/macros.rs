use crate::rpc::{
    AppendEntriesRequest, AppendEntriesResponse, Message, RPCMessage, RequestVoteRequest,
    RequestVoteResponse,
};

macro_rules! append_entries_request {
    //parameter:&self, entries:Vec<String>, peer:String("all")
    ($node:expr, $entries: expr, "all") => {
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
    ($node:expr, $entries: expr, $peer: expr) => {
        let AERMsg = RPCMessage::new(Message::AppendEntriesRequest(AppendEntriesRequest::new(
            $node.raft_info.current_term,
            $node.raft_info.node_id,
            $node.raft_info.last_applied,
            $node.raft_info.logs.last().unwrap().0,
            $entries,
            $node.raft_info.commit_index,
        )))
        .unwrap();
        $node
            .rpc
            .rpc_cs
            .send_to($peer.to_socket_addrs()?.next().unwrap(), &AERMsg)
            .unwrap();
    };
}

macro_rules! append_entries_response {
    //parameter:&self, success:bool, peer:String
    ($node:expr, $success: expr, $peer: expr) => {
        let AERMsg = RPCMessage::new(Message::AppendEntriesResponse(AppendEntriesResponse::new(
            $node.raft_info.current_term,
            $success,
        )))
        .unwrap();
        $node
            .rpc
            .rpc_cs
            .send_to($peer.to_socket_addrs()?.next().unwrap(), &AERMsg)
            .unwrap();
    };
}

macro_rules! request_vote_request {
    //parameter:&self (send to all)
    ($node:expr) => {
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

macro_rules! request_vote_response {
    //parameter:&self, success:bool
    ($node:expr, $vote_granted: expr, $peer: expr) => {
        let RVRMsg = RPCMessage::new(Message::RequestVoteResponse(RequestVoteResponse::new(
            $node.raft_info.current_term,
            $vote_granted,
        )))
        .unwrap();
        $node
            .rpc
            .rpc_cs
            .send_to($peer.to_socket_addrs()?.next().unwrap(), &RVRMsg)
            .unwrap();
    };
}
