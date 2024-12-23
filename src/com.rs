#![allow(dead_code)]

use anyhow::Result;
use std::{fmt::Debug, sync::Arc};
use tokio::sync::RwLock;

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
        self.broadcaster.send(message)?;
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
