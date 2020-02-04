use crossbeam_channel::{bounded, select, Receiver, Sender};
use rand::Rng;
use std::error::Error;
use std::io;
use std::sync::*;
use std::thread;
use std::time::Duration;


#[derive(Clone)]
struct Clock {
    canceller: Sender<()>,
    cancellee: Receiver<()>,
}

impl Clock {
    fn sleep(&self, duration: Duration) -> io::Result<()> {
        select! {
            recv(self.cancellee) -> _ => Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "Timer cancelled"),
            ),
            default(duration) => Ok(())
        }
    }
}

pub struct NodeTimer {
    notifier: Arc<Sender<()>>,
    pub receiver: Arc<Receiver<()>>,
    clock: Arc<Clock>,
    heartbeat_interval: Arc<Duration>,
}

impl NodeTimer {
    /// Create a [Timer](struct.Timer.html) and its associated [Canceller](struct.Canceller.html).
    pub fn new(interval: u32) -> Result<Self, Box<dyn Error>> {
        let (notifier, receiver) = bounded(0);
        let (canceller, cancellee) = bounded(0);

        Ok(NodeTimer {
            notifier: Arc::new(notifier),
            receiver: Arc::new(receiver),
            clock: Arc::new(Clock {
                canceller,
                cancellee,
            }),
            heartbeat_interval: Arc::new(Duration::from_millis(interval as u64)),
        })
    }

    pub fn run_elect(&self) {
        let clock = Arc::clone(&self.clock);
        let notifier = Arc::clone(&self.notifier);
        thread::spawn(move || {
            let mut interval = Duration::from_millis(rand::thread_rng().gen_range(150, 300));
            while clock.sleep(interval).is_err() {
                // timer.sleep return Ok(()) if the given time has elapsed
                // else return Err(...)
                interval = Duration::from_millis(rand::thread_rng().gen_range(150, 300));
            }
            notifier.try_send(());
        });
    }

    // must run timer before reset
    pub fn reset_elect(&self) {
        self.clock.canceller.try_send(());
    }

    // start heartbeat
    pub fn run_heartbeat(&self) {
        let clock = Arc::clone(&self.clock);
        let notifier = Arc::clone(&self.notifier);
        let heartbeat_interval = Arc::clone(&self.heartbeat_interval);

        thread::spawn(move || {
            while clock.sleep(*heartbeat_interval).is_ok() {
                notifier.try_send(());
            }
        });
    }

    // can be deleted, depend on how to change leader to follower
    pub fn stop_heartbeat(&self) {
        self.clock.canceller.try_send(());
    }
}
