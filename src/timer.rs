use crossbeam_channel::{unbounded, Receiver, Sender};
use cancellable_timer::*;
use rand::Rng;
use std::sync::Arc;
use std::thread;
use std::error::Error;
use std::time::Duration;

use crate::error::InitializationError;


pub struct NodeTimer {
    timer: Option<Timer>,
    canceller: Option<Canceller>,
    notifier: Arc<Sender<()>>,
    pub receiver: Option<Receiver<()>>,
    heartbeat_interval: Option<Duration>,
}

impl NodeTimer {
    pub fn new(interval: u32) -> Result<NodeTimer, Box<dyn Error>> {
        let (mut timer, canceller) = Timer::new2()?;
        let (tx, rx) = unbounded();
        return Ok(NodeTimer {
            timer: Some(timer),
            canceller: Some(canceller),
            notifier: Arc::new(tx),
            receiver: Some(rx),
            heartbeat_interval: Some(Duration::from_millis(interval as u64)),
        });
        Err(Box::new(InitializationError::TimerInitializationError))
    }

    // start election
    pub fn run_elect(&mut self) {
        if let Some(mut timer) = self.timer.take() {
            let timeout_notifier = Arc::clone(&self.notifier);
            thread::spawn(move || {
                let mut interval = Duration::from_millis(rand::thread_rng().gen_range(100, 500));
                while timer.sleep(interval).is_err() {
                    // timer.sleep return Ok(()) if the given time has elapsed
                    // else return Err(...) 
                    interval = Duration::from_millis(rand::thread_rng().gen_range(100, 500));
                }           
                
                timeout_notifier.send(());
                println!("election timeout after {} milliseconds", interval.as_millis());      
            });
        }
    }

    // must run timer before reset
    pub fn reset_elect(&mut self) {
        if let Some(canceller) = self.canceller.take() {
            canceller.cancel();
        }
    }


    // start heartbeat
    pub fn run_heartbeat(&mut self) {
        if let Some(mut timer) = self.timer.take() {
            let timeout_notifier =  Arc::clone(&self.notifier);
            if let Some(heartbeat_interval) = self.heartbeat_interval.take() {
                thread::spawn(move || {
                    while timer.sleep(heartbeat_interval).is_ok() {
                        timeout_notifier.send(());
                        println!("heartbeat after {} milliseconds", heartbeat_interval.as_millis());                
                    }
                });
            } 
        }
    }

    // can be deleted, depend on how to change leader to follower
    pub fn stop_heartbeat(&mut self) {
        if let Some(canceller) = self.canceller.take() {
            canceller.cancel();
        }
    }
}