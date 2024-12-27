use std::{
    collections::HashMap,
    fmt::Debug,
    future::Future,
    pin::Pin,
    sync::{Arc, LazyLock},
};

use crate::{irc_parser::IrcMessage, twitch_client::TWITCH_MSG, Args};
use anyhow::{Error, Result};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

pub static BOT_COMMANDS: LazyLock<BotCommands> = LazyLock::new(|| BotCommands::new());

pub static COMMAND_PREFIX: &str = "!!";
static BOT_COMMAND_DIR: &str = "bot_commands";
static COMMANDS_FILE_EXT: &str = "toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMessage {
    pub timestamp: i64,
    pub sender: String,
    pub message: String,
}

type BotCommandFn = Box<
    dyn Fn(IrcMessage) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> + Send + Sync,
>;

impl BotCommands {
    pub fn new() -> Self {
        BotCommands {
            commands: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_command(&self, trigger: String, command: BotCommandFn) {
        self.commands.write().await.insert(trigger.clone(), command);
    }

    pub async fn run_command(&self, command: &str, message: IrcMessage) -> Result<()> {
        if let Some(func) = self.commands.read().await.get(command) {
            func(message).await?;
        }
        Ok(())
    }
}

pub struct BotCommands {
    commands: Arc<RwLock<HashMap<String, BotCommandFn>>>,
}

pub async fn start(_args: Args) -> Result<()> {
    let mut test_broadcast_rx = TWITCH_MSG.subscribe_broadcast().await;

    BOT_COMMANDS
        .add_command(
            "list".into(),
            Box::new(|irc_message| Box::pin(list_all_commands(irc_message))),
        )
        .await;

    BOT_COMMANDS
        .add_command(
            "test".into(),
            Box::new(|irc_message| Box::pin(test_command(irc_message))),
        )
        .await;

    // Read all broadcasted commands from Twitch_client
    while let Ok(ret_val) = test_broadcast_rx.recv().await {
        match ret_val.context.command.as_str() {
            "PRIVMSG" if ret_val.payload.starts_with(COMMAND_PREFIX) => {
                let command = ret_val
                    .payload
                    .split_whitespace()
                    .next()
                    .unwrap()
                    .trim_start_matches("!!");
                BOT_COMMANDS.run_command(command, ret_val.clone()).await?;
            }
            _ => {}
        };
    }

    Ok(())
}

pub async fn test_command(message: IrcMessage) -> Result<()> {
    let ret_val = format!(
        "Hi there {} this is the reply to your test message",
        message.context.sender
    );
    TWITCH_MSG.send(ret_val).await?;
    Ok(())
}

pub async fn list_all_commands(_message: IrcMessage) -> Result<()> {
    BOT_COMMANDS
        .commands
        .read()
        .await
        .iter()
        .for_each(|(trigger, _)| println!("{}", trigger));
    Ok(())
}
