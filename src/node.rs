use crate::service;
use crate::ClusterInfo;
use std::thread;

// Role of a Node
enum Role {
    Follower,
    Candidate,
    Leader,
}

// Log content
struct Log {
    // For each log, there is a state such as "x->1", which
    // sets the value_id x to value_content 1
    value_id: u32,
    value_content: i32,
    rec_term: u32,
}

// Node represents a single server
pub struct Node {
    id: u32,
    current_term: u32,
    voted_for: u32,
    logs: Vec<Log>,
    commit_index: u32,
    last_applied: u32,
    next_index: Vec<u32>,
    match_index: Vec<u32>,
    role: Role,
    port: u16,
    cluster_info: ClusterInfo,
}

impl Node {
    pub fn new(id: u32, cluster_info: ClusterInfo) -> Node {
        Node {
            id,
            current_term: 0,
            voted_for: 0,
            logs: Vec::<Log>::new(),
            commit_index: 0,
            last_applied: 0,
            next_index: Vec::<u32>::new(),
            match_index: Vec::<u32>::new(),
            role: Role::Follower,
            port: cluster_info.get_node_port(id),
            cluster_info,
        }
    }

    fn start_rpc_server(port: u16) {
        service::init_server(port).unwrap();
    }

    pub fn run(self) {
        let server_port = self.port;
        let rpc_server_thread = thread::spawn(move || {
            println!("Running RPC Server on 127.0.0.1:{}", server_port);
            Node::start_rpc_server(server_port);
        });

        rpc_server_thread.join().unwrap();
    }

    fn request_a_vote(self, node_id: u32) {
        let port = self.cluster_info.get_node_port(node_id);
        println!("Calling RPC Server on 127.0.0.1:{}", port);
        service::request_a_vote(port).unwrap();
    }

    fn append_entries(self) {
        // To-do: Implement append_entries method
    }
}
