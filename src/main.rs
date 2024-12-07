#![allow(dead_code)]
use com::CommBus;
use futures::stream::FuturesUnordered;
use std::sync::Arc;
use tokio::{sync::RwLock, task::JoinHandle};
use tokio_stream::StreamExt;
mod colors;
mod com;
mod config_manager;
mod irc_parser;
mod macros;
mod tts;
mod tts_player;
mod twitch_client;
mod users_manager;

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
struct Args {
    bot_info: BOTInfo,
    com_bus: CommBus,
}

#[tokio::main]
async fn main() {
    let bot_info = BOTInfo::default();

    let args = Args {
        com_bus: CommBus::new(),
        bot_info,
    };

    let mut manager = TaskManager::new();
    manager.add_task("TWITCH", twitch_client::start(args.clone()));
    manager.add_task("TTS", tts::start(args.clone()));
    manager.add_task("TTS_PLAYER", tts_player::start(args.clone()));
    manager.add_task("USERS", users_manager::start(args.clone()));
    manager.run().await;
}
