use super::rpc::{Message, RPCMessage, RequestVoteRequest, RPCCS};
use crossbeam_channel::{select, unbounded};
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::thread;

#[test]
fn rpc_send_rec() {
    let socket_addr = Arc::new("127.0.0.1:2995".to_socket_addrs().unwrap().next().unwrap());
    let peer_list: Vec<SocketAddr> = vec![*Arc::clone(&socket_addr)];
    let rpc_cs = Arc::new(RPCCS::new(*socket_addr, peer_list).unwrap());
    let (rpc_notifier, rpc_receiver) = unbounded();

    let rpc_client = Arc::clone(&rpc_cs);
    thread::spawn(move || rpc_client.start_listener(rpc_notifier).unwrap());

    let msg_to_send = RPCMessage::new(Message::RequestVoteRequest(RequestVoteRequest::new(
        0, 0, 0, 0,
    )))
    .unwrap();

    rpc_cs.send_all(&msg_to_send).unwrap();

    select! {
        recv(rpc_receiver) -> msg => {
            assert_eq!(msg_to_send, msg.unwrap());
        }
    }
}
