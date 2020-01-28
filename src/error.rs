use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum InitializationError {
    NodeInitializationError,
    RPCInitializationError,
    TimerInitializationError,
    RaftInitializationError,
}

impl fmt::Display for InitializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            InitializationError::NodeInitializationError => write!(f, "Initializing Node Error"),
            InitializationError::RPCInitializationError => write!(f, "Initializing RPC Error"),
            InitializationError::TimerInitializationError => write!(f, "Initializing Timer Error"),
            InitializationError::RaftInitializationError => write!(f, "Initializing Raft Error"),
        }
    }
}

impl Error for InitializationError {}
