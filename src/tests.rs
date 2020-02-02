use super::rpc::{Message, RPCMessage, RequestVoteRequest, RPCCS};
use super::timer::NodeTimer;
use crossbeam_channel::{select, unbounded};
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[cfg(test)]
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

#[test]
fn timer_run_elect() {
    let timer = NodeTimer::new(5).unwrap();
    timer.run_elect();
    timer.receiver.recv().unwrap();
}

#[test]
fn timer_reset_elect() {
    let timer = NodeTimer::new(5).unwrap();
    timer.run_elect();
    timer.reset_elect();
    timer.receiver.recv().unwrap();
}

#[test]
fn timer_run_heartbeat() {
    let timer = NodeTimer::new(5).unwrap();
    timer.run_heartbeat();

    let mut count = 0;
    while count != 10 {
        select! {
            recv(timer.receiver) -> _ => count += 1,
        }
    }
    assert_eq!(count, 10);
}

// #[test]
// fn timer_stop_heartbeat() -> Result<(), String> {
//     let timer = NodeTimer::new(5).unwrap();
//     timer.run_heartbeat();
//     timer.stop_heartbeat();

//     select! {
//         recv(timer.receiver) -> _ => Err(String::from("stop heartbeat failure")),
//         default(Duration::from_millis(5)) => Ok(()),
//     }
// }
