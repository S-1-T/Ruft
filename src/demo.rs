pub enum Role {
    Follower,
    Candidate,
    Leader,
}

pub trait raft {
    fn handle_append_entries();
    fn handle_request_vote();
    ...
}

// 以 Follower 为例，其余同理
pub struct Follower {
    // Follower 独有的数据结构
}
impl Follower {
    // Follower 独有的函数
}
impl raft for Follower {
    // raft
}


struct Node {
    // 公有数据结构
    // ...
    role: Box<raft>,
    // ...
}
impl Node {
    fn new(&self) ->  {
        
    }

    fn change_to(&self, rolename: Role) -> Box<raft> {
        match rolename {
            Role::Follower => self.role = Box::new(Follower::new());
            Role::Candidate => self.role = Box::new(Candidate::new());
            Role::Leader => self.role = Box::new(Leader::new());
        }
    }
}

struct Server {
    
}




