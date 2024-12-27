#![allow(dead_code)]
#![feature(async_closure)]

use futures::stream::FuturesUnordered;
use std::sync::Arc;
use tokio::{sync::RwLock, task::JoinHandle};
use tokio_stream::StreamExt;

pub mod audio_player;
pub mod colors;
pub mod com;
pub mod commands;
pub mod config_manager;
pub mod irc_parser;
pub mod macros;
pub mod tts;
pub mod twitch_client;
pub mod users_manager;

#[derive(Debug, Clone, Default)]
pub struct BOTInfo {
    name: Arc<RwLock<String>>,
    main_channel: Arc<RwLock<String>>,
}

impl BOTInfo {
    pub async fn set_name(&self, name: &str) {
        *self.name.write().await = name.to_string();
    }

    pub async fn set_main_channel(&self, main_channel: &str) {
        *self.main_channel.write().await = main_channel.to_string();
    }

    pub async fn get_name(&self) -> String {
        self.name.read().await.clone()
    }

    pub async fn get_main_channel(&self) -> String {
        self.main_channel.read().await.clone()
    }
}

struct TaskManager {
    tasks: FuturesUnordered<JoinHandle<Result<(), anyhow::Error>>>,
}

impl TaskManager {
    fn new() -> Self {
        Self {
            tasks: FuturesUnordered::new(),
        }
    }

    fn add_task<F>(&mut self, _name: &'static str, task: F)
    where
        F: std::future::Future<Output = Result<(), anyhow::Error>> + Send + 'static,
    {
        self.tasks.push(tokio::spawn(task));
    }

    async fn run(&mut self) {
        while let Some(result) = self.tasks.next().await {
            match result {
                Ok(Ok(_)) => println!("Task completed successfully"),
                Ok(Err(e)) => ErrorPrint!("Task failed: {:?}", e),
                Err(e) => ErrorPrint!("Task panicked: {:?}", e),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Args {}

#[tokio::main]
async fn main() {
    let args = Args {};

    let mut task_manager = TaskManager::new();
    task_manager.add_task("TWITCH", twitch_client::start(args.clone()));
    task_manager.add_task("TTS", tts::start(args.clone()));
    task_manager.add_task("TTS_PLAYER", audio_player::start(args.clone()));
    task_manager.add_task("COMMANDS", commands::start(args.clone()));
    task_manager.run().await;
}
