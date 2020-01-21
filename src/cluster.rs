pub struct ClusterInfo {
    // For convenience, the node's port will start from
    // strat_port and increases by node's id
    strat_port: u16,
    node_number: u32,
    majority_number: u32,
    heartbeat_interval: u32,
}

impl ClusterInfo {
    pub fn new(strat_port: u16, node_number: u32, heartbeat_interval: u32) -> ClusterInfo {
        let majority_number = (node_number - 1) / 2 + 1;

        ClusterInfo {
            strat_port,
            node_number,
            majority_number,
            heartbeat_interval,
        }
    }

    pub fn get_node_port(&self, node_id: u32) -> u16 {
        self.strat_port + node_id as u16
    }
}
