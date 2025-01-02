#![allow(dead_code)]
#![feature(async_closure)]

use anyhow::{Error, Result};
use std::{future::Future, pin::Pin, process::exit, sync::Arc};
use tokio::sync::RwLock;

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
pub struct BotTask {
    name: String,
    task_fn: Arc<dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> + Send + Sync>,
    restarts: u8,
    max_restarts: u8,
}

impl BotTask {
    pub fn new(
        name: String,
        task_fn: impl Fn() -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>
            + Send
            + Sync
            + 'static,
        max_restarts: u8,
    ) -> Self {
        BotTask {
            name,
            task_fn: Arc::new(task_fn),
            restarts: 0,
            max_restarts,
        }
    }

    pub async fn run(self) {
        let task_fn = self.task_fn.clone();
        tokio::spawn(async move {
            let mut restarts = 0;
            while restarts <= self.max_restarts {
                match (task_fn)().await {
                    Ok(_) => {
                        println!("{} task finished successfully!", self.name);
                        break;
                    }
                    Err(err) => {
                        println!("{} task failed: {:?}", self.name, err);
                        restarts += 1;
                        if restarts > self.max_restarts {
                            println!("{} task reached the maximum number of restarts.", self.name);
                            println!("Exiting...");
                            exit(1);
                        }
                    }
                }
                println!(
                    "Restarting {} task... {}/{}",
                    self.name, restarts, self.max_restarts
                );
            }
        });
    }
}

pub struct TaskManager {
    tasks: Vec<BotTask>,
}

impl TaskManager {
    pub fn new() -> Self {
        TaskManager { tasks: Vec::new() }
    }

    pub fn add_task(&mut self, task: BotTask) {
        self.tasks.push(task);
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Args {}

#[tokio::main]
async fn main() {
    let args = Args {};
    let mut task_manager = TaskManager::new();

    let twitch_task = BotTask::new(
        "Twitch Client".into(),
        move || Box::pin(twitch_client::start(args.clone())),
        5,
    );

    let tts_task = BotTask::new("TTS".into(), move || Box::pin(tts::start(args.clone())), 5);

    let commands_task = BotTask::new(
        "Commands".into(),
        move || Box::pin(commands::start(args.clone())),
        5,
    );

    let audio_player_task = BotTask::new(
        "Audio Player".into(),
        move || Box::pin(audio_player::start(args.clone())),
        5,
    );

    task_manager.add_task(twitch_task);
    task_manager.add_task(tts_task);
    task_manager.add_task(commands_task);
    task_manager.add_task(audio_player_task);

    for task in task_manager.tasks {
        task.run().await;
    }

    tokio::signal::ctrl_c().await.unwrap();
}

async fn run_task<F, Fut>(task_name: String, task_fn: F, args: Args)
where
    F: Fn(Args) -> Fut + Send + 'static,
    Fut: Future<Output = Result<(), Error>> + Send + 'static,
{
    tokio::spawn(async move {
        loop {
            match task_fn(args.clone()).await {
                Ok(t) => {
                    println!("{} task finished successfully {:?}", task_name, t);
                }
                Err(err) => {
                    println!(
                        "{} task failed with error: {:?}. Restarting...",
                        task_name, err
                    );
                }
            }
        }
    });
}
