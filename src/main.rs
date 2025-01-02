#![allow(dead_code)]

use anyhow::{Error, Result};
use futures::executor::block_on;
use std::{fmt::Display, future::Future, pin::Pin, sync::Arc};
use tokio::sync::{Notify, RwLock};

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

#[derive(Clone)]
pub struct BotTask {
    name: String,
    task_fn: Arc<dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> + Send + Sync>,
    restarts: Arc<RwLock<u8>>,
    max_restarts: u8,
    current_join_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
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
            restarts: Arc::new(RwLock::new(0)),
            max_restarts,
            current_join_handle: Arc::new(RwLock::new(None)),
        }
    }
}

impl Display for BotTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        block_on(async {
            write!(
                f,
                "{} task, max restarts {}, actual restarts {}",
                self.name,
                self.max_restarts,
                self.restarts.read().await
            )
        })
    }
}

impl std::fmt::Debug for BotTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        block_on(async {
            write!(
                f,
                "{} task, max restarts {}, actual restarts {}",
                self.name,
                self.max_restarts,
                self.restarts.read().await
            )
        })
    }
}

#[derive(Debug)]
pub struct TaskManager {
    exit_conditions: Arc<Notify>,
    tasks: Vec<BotTask>,
}

impl TaskManager {
    pub fn new() -> Self {
        TaskManager {
            exit_conditions: Arc::new(Notify::new()),
            tasks: Vec::new(),
        }
    }

    pub fn add_task(&mut self, task: BotTask) {
        self.tasks.push(task);
    }

    pub async fn run(&mut self) {
        for task in &self.tasks {
            let exit_condition = self.exit_conditions.clone();
            let task_clone = task.clone();
            let jh = tokio::spawn(async move {
                loop {
                    println!(
                        "{}\n{} {} task...\n{}",
                        "#".repeat(100),
                        if *task_clone.restarts.read().await == 0 {
                            "Starting"
                        } else {
                            "Restarting"
                        },
                        &task_clone,
                        "#".repeat(100)
                    );
                    match (task_clone.task_fn)().await {
                        Ok(_) => {
                            println!("{} task finished successfully!", task_clone.name);
                            break;
                        }
                        Err(err) => {
                            println!("{} task failed: {:?}", task_clone.name, err);
                        }
                    }
                    if *task_clone.restarts.write().await == task_clone.max_restarts {
                        break;
                    }
                    *task_clone.restarts.write().await += 1;
                }
                println!(
                    "{} task reached the maximum number of restarts.",
                    task_clone.name
                );
                exit_condition.notify_waiters();
            });

            task.current_join_handle.write().await.replace(jh);
        }
    }

    pub async fn statics(&self) -> String {
        let mut statics = String::new();
        for task in &self.tasks {
            statics.push_str(&format!("{}\n", task));
        }
        statics
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
        2,
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

    task_manager.run().await;

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("\nReceived Ctrl-C signal. Exiting...");
        }

        _ = task_manager.exit_conditions.notified() => {
            println!("Exiting...");
        }
    }

    println!("Task status: {}", task_manager.statics().await);
}

// async fn run_task<F, Fut>(task_name: String, task_fn: F, args: Args)
// where
//     F: Fn(Args) -> Fut + Send + 'static,
//     Fut: Future<Output = Result<(), Error>> + Send + 'static,
// {
//     tokio::spawn(async move {
//         loop {
//             match task_fn(args.clone()).await {
//                 Ok(t) => {
//                     println!("{} task finished successfully {:?}", task_name, t);
//                 }
//                 Err(err) => {
//                     println!(
//                         "{} task failed with error: {:?}. Restarting...",
//                         task_name, err
//                     );
//                 }
//             }
//         }
//     });
// }
