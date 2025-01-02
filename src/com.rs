#![allow(dead_code)]

use anyhow::Result;
use std::{collections::VecDeque, fmt::Debug, sync::Arc};
use tokio::sync::{Notify, RwLock};

#[derive(Debug, Clone)]

pub struct MsgChannel<BM, SM>
where
    BM: Sync + Send + Clone + 'static,
    SM: Sync + Send + Clone + 'static,
{
    name: String,
    broadcaster: tokio::sync::broadcast::Sender<BM>,
    receiver: Arc<tokio::sync::RwLock<tokio::sync::mpsc::Receiver<SM>>>,
    sender: tokio::sync::mpsc::Sender<SM>,
}

impl<BM, SM> MsgChannel<BM, SM>
where
    BM: Sync + Send + Clone + Debug + 'static,
    SM: Sync + Send + Clone + Debug + 'static,
{
    pub fn new(name: String, capacity: usize) -> Self {
        let (broadcaster_tx, _) = tokio::sync::broadcast::channel(capacity);
        let (tx, rx) = tokio::sync::mpsc::channel(capacity);
        MsgChannel {
            name,
            broadcaster: broadcaster_tx,
            sender: tx,
            receiver: Arc::new(RwLock::new(rx)),
        }
    }

    pub fn init(&self) -> &Self {
        println!("Channel {} initialized", self.name);
        self
    }

    pub async fn send_broadcast(&self, message: BM) -> Result<()> {
        if self.broadcaster.receiver_count() > 0 {
            self.broadcaster.send(message)?;
        }
        Ok(())
    }

    pub async fn subscribe_broadcast(&self) -> tokio::sync::broadcast::Receiver<BM> {
        self.broadcaster.subscribe()
    }

    pub async fn send(&self, message: SM) -> Result<()> {
        self.sender.send(message).await?;
        Ok(())
    }

    pub async fn recv(&self) -> Result<SM> {
        self.receiver.write().await.recv().await.ok_or_else(|| {
            anyhow::anyhow!(
                "Error: Failed to receive message from channel {}",
                self.name
            )
        })
    }
}

#[derive(Debug, Clone)]
pub struct MSGQueue<T>
where
    T: Sync + Send + Clone + Debug + 'static,
{
    queue: Arc<RwLock<VecDeque<T>>>,
    notify: Arc<tokio::sync::Notify>,
}

impl<T> MSGQueue<T>
where
    T: Sync + Send + Clone + Debug + 'static,
{
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn push_back(&self, payload: T) {
        self.queue.write().await.push_back(payload);
        println!("Audio buffer {:#?}", self.queue.read().await.len());
        self.notify.notify_waiters();
    }

    pub async fn next(&self) -> Option<T> {
        loop {
            println!("Audio buffer {:#?}", self.queue.read().await.len());
            if let Some(value) = self.queue.write().await.pop_front() {
                return Some(value);
            }
            self.notify.notified().await;
        }
    }

    pub async fn next_error(&self) -> Result<T> {
        loop {
            println!("Audio buffer {:#?}", self.queue.read().await.len());
            if let Some(value) = self.queue.write().await.pop_front() {
                return Ok(value);
            }
            self.notify.notified().await;
        }
    }

    pub async fn len(&self) -> usize {
        self.queue.read().await.len()
    }
}
