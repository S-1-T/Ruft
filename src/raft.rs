use std::error::Error;

#[macro_use]
use crate::{append_entries_request, append_entries_response,
            request_vote_request, request_vote_response};

// Role of a Node
#[derive(PartialEq, Copy, Clone)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
}

pub trait raft {
    fn handle_append_entries_request(&self);
    fn handle_append_entries_response(&self);
    fn handle_request_vote_request(&self);
    fn handle_request_vote_response(&self);
    fn handle_timeout(&self);
}

pub struct Follower {}
impl Follower {
    pub fn new() -> Result<Box<Follower>, Box<dyn Error>> {
        Ok(Box::new(Follower {}))
    }
    // Follower 独有的函数
}
impl raft for Follower {
    fn handle_append_entries_request(&self) {

    }

    fn handle_append_entries_response(&self) {

    }

    fn handle_request_vote_request(&self) {

    }
    
    fn handle_request_vote_response(&self) {

    }

    fn handle_timeout(&self) {

    }
}

pub struct Candidate {
    votes : u8,
}
impl Candidate {
    pub fn new() -> Result<Box<Candidate>, Box<dyn Error>> {
        Ok(Box::new(Candidate {votes: 0}))
    }

}
impl raft for Candidate {
    fn handle_append_entries_request(&self) {

    }

    fn handle_append_entries_response(&self) {

    }

    fn handle_request_vote_request(&self) {

    }
    
    fn handle_request_vote_response(&self) {

    }

    fn handle_timeout(&self) {
        
    }
}

pub struct Leader {
    next_index: Vec<u32>,
    match_index: Vec<u32>,
}
impl Leader {
    pub fn new() -> Result<Box<Leader>, Box<dyn Error>> {
        Ok(Box::new(Leader {
            next_index: Vec::<u32>::new(),
            match_index: Vec::<u32>::new(),
        }))
    }
}
impl raft for Leader {
    fn handle_append_entries_request(&self) {

    }

    fn handle_append_entries_response(&self) {

    }

    fn handle_request_vote_request(&self) {

    }
    
    fn handle_request_vote_response(&self) {

    }

    fn handle_timeout(&self) {
        
    }
}

pub struct RaftInfo {
    pub node_id: u32,
    pub role: Box<dyn raft>,
    pub current_term: u32,
    pub voted_for: u32,
    pub logs: Vec<(u32, String)>,
    pub commit_index: u32,
    pub last_applied: u32,
}