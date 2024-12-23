use std::collections::HashMap;

use crate::{irc_parser::IrcMessage, twitch_client::TWITCH_MSG, Args};

use anyhow::Result;

use serde::{Deserialize, Serialize};
use tokio::fs::{self, read_dir};

pub static COMMAND_PREFIX: &str = "!!";
static BOT_COMMAND_DIR: &str = "bot_commands";
static COMMANDS_FILE_EXT: &str = "toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMessage {
    pub timestamp: i64,
    pub sender: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

struct BotCommand {
    trigger: String,
    response: Option<String>,
    error_message: Option<String>,
}

impl BotCommand {
    fn new(trigger: String, response: Option<String>, error_message: Option<String>) -> Self {
        BotCommand {
            trigger,
            response,
            error_message,
        }
    }

    fn parse(&mut self, irc_msg: &IrcMessage) -> Self {
        if let Some(response) = &self.response {
            self.response = Some(response.replace("{sender}", &irc_msg.context.sender));
        }
        self.clone()
    }
}

struct BotCommands {
    commands: HashMap<String, BotCommand>,
}

impl BotCommands {
    fn new() -> Self {
        BotCommands {
            commands: HashMap::new(),
        }
    }

    fn add_command(&mut self, command: BotCommand) {
        self.commands.insert(command.trigger.clone(), command);
    }

    fn get_command(&mut self, trigger: &str) -> Option<&mut BotCommand> {
        println!("Trigger: {}", trigger);
        self.commands.get_mut(trigger)
    }

    fn list_commands(&self) -> String {
        self.commands
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .join(", ")
    }
}

pub async fn start(_args: Args) -> Result<()> {
    let mut test_broadcast_rx = TWITCH_MSG.subscribe_broadcast().await;
    let mut commands = load_all_commands().await?;

    println!("Commands: {}", commands.list_commands());

    while let Ok(ret_val) = test_broadcast_rx.recv().await {
        match ret_val.context.command.as_str() {
            "PRIVMSG" if ret_val.payload.starts_with(COMMAND_PREFIX) => {
                let command = ret_val
                    .payload
                    .split_whitespace()
                    .next()
                    .unwrap()
                    .trim_start_matches("!!");
                println!("##############Command: {}", command);

                commands.get_command(command).map(|c| {
                    let response = c.parse(&ret_val);
                    println!("###################Response: {:?}", response);
                });
            }

            _ => {}
        };
    }

    Ok(())
}

async fn load_all_commands() -> Result<BotCommands> {
    let mut commands = BotCommands::new();
    let mut dir_content = read_dir(BOT_COMMAND_DIR).await?;

    while let Some(file) = dir_content.next_entry().await? {
        if file.path().extension().unwrap_or_default() == COMMANDS_FILE_EXT {
            let command = fs::read_to_string(file.path()).await?;
            match toml::from_str::<BotCommand>(&command) {
                Ok(c) => commands.add_command(c),
                Err(e) => eprintln!("Error parsing command file: {:?}", e),
            }
        }
    }
    Ok(commands)
}
