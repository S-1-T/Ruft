use ruft::{ClusterInfo, Node};

fn main() {
    println!("Hello, Ruft!");

    let start_port: u16 = 4399;
    let node_num: u32 = 5;
    let heartbeat_interval: u32 = 5;
    let current_node_id = 0;

    let cluster_info = ClusterInfo::new(start_port, node_num, heartbeat_interval);
    let node = Node::new(current_node_id, cluster_info);

    node.run();
}
