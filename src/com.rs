#![allow(dead_code)]

use std::{ any::{ type_name, Any }, collections::HashMap, fmt::Debug, sync::Arc };
use anyhow::Result;
use tokio::sync::{ mpsc::{ self }, RwLock };

use crate::ErrorPrint;

type CommQueue<T> = mpsc::Sender<T>;

#[derive(Debug, Clone)]
pub struct CommSubscriber {
    pub p_type: &'static str,
    pub sender: Arc<dyn Any + Send + Sync>,
}

impl CommSubscriber {
    pub fn new<T>(receiver: mpsc::Sender<T>) -> Self where T: Any + Send + Clone + 'static {
        CommSubscriber { p_type: type_name::<T>(), sender: Arc::new(receiver) }
    }
}

#[derive(Debug, Clone)]
pub struct CommBus {
    // pub queues: Arc<RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>>,
    pub queues: Arc<RwLock<HashMap<String, CommSubscriber>>>,
}

impl CommBus {
    pub fn new() -> Self {
        CommBus {
            queues: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_queue<T>(&self, name: String, queue: CommQueue<T>)
        where T: Any + Send + Clone + 'static
    {
        // self.queues.write().await.insert(name, Arc::new(queue) as Arc<dyn Any + Send + Sync>);
        self.queues.write().await.insert(name, CommSubscriber::new::<T>(queue));
    }

    pub async fn get_queue<T>(&self, name: &'static str) -> Option<CommQueue<T>>
        where T: Any + Send + Clone + 'static
    {
        if let Some(queue) = self.queues.read().await.get(name) {
            let queue = queue.clone();
            if let Some(queue) = queue.sender.downcast_ref::<CommQueue<T>>() {
                return Some(queue.clone());
            } else {
                ErrorPrint!("Error: Downcasting queue failed. Discarding the queue for {} module.", name);
                ErrorPrint!(
                    "Supplied type is {:?}, expected type is {:?}, message will be discarded.",
                    type_name::<T>(),
                    queue.p_type
                );
            }
        }
        None
        // self.queues
        //     .read().await
        //     .get(name)
        //     .cloned()
        //     .map(|arc_any| { arc_any.clone().downcast::<CommQueue<T>>().ok() })
        //     .flatten()
        //     .as_deref()
        //     .cloned()
    }

    pub async fn subscribe<T>(&self, name: &'static str) -> mpsc::Receiver<T>
        where T: Any + Send + Clone + 'static
    {
        let (sender, receiver) = mpsc::channel(100);
        self.add_queue(name.to_string(), sender).await;
        receiver
    }

    pub async fn send<T>(&self, name: &'static str, message: T) -> Result<()>
        where T: Any + Send + Clone + Sync + 'static
    {
        if let Some(queue) = self.get_queue::<T>(name).await {
            queue.send(message).await.unwrap_or_else(|e| {
                ErrorPrint!(
                    "Error: Sending to {} module {:?}. Removing the relative message queue.",
                    name,
                    e
                );
                futures::executor::block_on(async {
                    self.queues.write().await.remove(name);
                });
            });
        }
        Ok(())
    }
}

fn get_type_name<T>(_value: &T) -> &'static str {
    std::any::type_name::<T>()
}
