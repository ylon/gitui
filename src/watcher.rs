use anyhow::{anyhow, Result};
use crossbeam_channel::{unbounded, Sender};
use notify::{
    DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::{
    thread,
    time::{Duration, Instant},
};

pub struct RepoWatcher {
    receiver: crossbeam_channel::Receiver<()>,
    #[allow(dead_code)]
    watcher: RecommendedWatcher,
}

static RECEIVE_TIMEOUT: Duration = Duration::from_millis(500);
static SEND_DELAY: Duration = Duration::from_millis(800);

impl RepoWatcher {
    pub fn new() -> Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher: RecommendedWatcher =
            Watcher::new(tx, Duration::from_secs(2))?;
        watcher.watch(".", RecursiveMode::Recursive)?;

        let (out_tx, out_rx) = unbounded();

        thread::spawn(move || Self::buffered_forwarder(rx, out_tx));

        Ok(Self {
            watcher,
            receiver: out_rx,
        })
    }

    ///
    pub fn receiver(&self) -> crossbeam_channel::Receiver<()> {
        self.receiver.clone()
    }

    #[allow(clippy::needless_pass_by_value)]
    fn buffered_forwarder(
        receiver: std::sync::mpsc::Receiver<DebouncedEvent>,
        sender: Sender<()>,
    ) -> Result<()> {
        let mut pending = None;

        loop {
            match receiver.recv_timeout(RECEIVE_TIMEOUT) {
                Err(
                    std::sync::mpsc::RecvTimeoutError::Disconnected,
                ) => return Err(anyhow!("channel disconnected")),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => (),
                Ok(ev) => {
                    log::debug!("notify: {:?}", ev);
                    if pending.is_none() {
                        pending = Some(Instant::now());
                    }
                }
            }

            if let Some(pending_time) = pending {
                if pending_time.elapsed() > SEND_DELAY {
                    log::debug!("notify forwarded");
                    sender.send(()).expect("send error");

                    pending = None;
                }
            }
        }
    }
}
