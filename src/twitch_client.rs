#![allow(dead_code)]

use crate::colors::Colorize;

use crate::com::MsgChannel;
use crate::config_manager::filename;
use crate::config_manager::ConfigManager;
use crate::irc_parser;

use crate::irc_parser::IrcMessage;
use crate::Args;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::Duration;
use std::vec;
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::Message;

use futures::{pin_mut, SinkExt, StreamExt};

pub static TWITCH_MSG: LazyLock<MsgChannel<IrcMessage, String>> =
    LazyLock::new(|| MsgChannel::new("TwitchMsg".into(), 100));

pub static BOT_INFO: LazyLock<BOTInfo> = LazyLock::new(|| BOTInfo::new());

#[derive(Debug, Clone, Default)]
pub struct BOTInfo {
    name: Arc<RwLock<String>>,
    main_channel: Arc<RwLock<String>>,
}

impl BOTInfo {
    pub fn new() -> Self {
        BOTInfo {
            name: Arc::new(RwLock::new("".into())),
            main_channel: Arc::new(RwLock::new("".into())),
        }
    }

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TwitchClient {
    file_name: String,
    pub server_address: String,
    pub nick: String,
    pub token: String,
    pub channel: String,
    pub log_level: String,
    pub anti_idle: i32,
}

impl ConfigManager for TwitchClient {}

impl Default for TwitchClient {
    fn default() -> Self {
        TwitchClient {
            file_name: filename("twitch_client"),
            server_address: "wss://irc-ws.chat.twitch.tv:443".into(),
            nick: "justinfan123".into(),
            token: "oauth:1234567890".into(),
            channel: "icsboyx".into(),
            log_level: "info".into(),
            anti_idle: 180,
        }
    }
}

trait WsMessageHandler {
    fn to_ws_text(&self) -> Message;
}

impl<T: std::fmt::Display> WsMessageHandler for T {
    fn to_ws_text(&self) -> Message {
        println!("{} {}", "[TX][RAW]".blue(), self);
        Message::text(self.to_string())
    }
}

pub async fn start(_args: Args) -> Result<()> {
    println!("[DEBUG] Starting Twitch Client");

    // Load twitch Client configuration or use default values and write to config file
    let twitch_client_config = TwitchClient::load_config::<TwitchClient>(TwitchClient::default())?;

    let server_address = twitch_client_config.server_address;
    let user_token = twitch_client_config.token;
    let user_nick = twitch_client_config.nick;
    let user_channel = twitch_client_config.channel;

    let (ws_stream, _response) = tokio_tungstenite::connect_async(server_address).await?;
    let (mut write, mut read) = ws_stream.split();

    for message in twitch_auth(&user_token, &user_nick, &user_channel) {
        write.send(message).await?;
    }

    let ping_interval = tokio::time::interval(Duration::from_secs(180));

    pin_mut!(ping_interval);

    loop {
        tokio::select! {
              _ = ping_interval.tick() => {
                  let payload = "PING :tmi.twitch.tv";
                  write.send(payload.to_ws_text()).await?;
                  }

              Some(line) = read.next() => {
                  if let Ok(line) = line {
                      let lines = line.to_text().unwrap().trim_end_matches("\r\n").split("\r\n");
                      for line in lines {
                          let payload = line;
                          println!("{}{} ","[RX][RAW] ".magenta(), payload);
                          let irc_message = irc_parser::parse_message(&payload.to_string());
                          TWITCH_MSG.send_broadcast(irc_message.clone()).await?;
                          match irc_message.context.command.as_str() {
                              "001" => {
                                  println!("{}{} ","[RX][RAW] ".magenta(), payload);
                                  println!("[DEBUG] Bot {}, connected to Twitch.", irc_message.context.destination);
                                  BOT_INFO.set_name(&irc_message.context.destination).await;
                                  BOT_INFO.set_main_channel(&user_channel).await;
                                  println!("[DEBUG] Bot Info: {:?}", BOT_INFO);
                              }
                              "PING" => {
                                  write.send("PONG :tmi.twitch.tv".to_ws_text()).await?;
                              }
                              _ => {
                                  // TODO: Add more commands
                              }
                              }
                  }
              }


        }

              ret_val = TWITCH_MSG.recv() => {
                if let Ok(ret_val) = ret_val {
                    println!("[DEBUG] Received message: {:?}", ret_val);
                    write.send(format!("PRIVMSG #{} :{}", user_channel, ret_val).to_ws_text()).await?;
              }}

        }
    }
}

fn twitch_auth(user_token: &String, user_nick: &String, user_channel: &String) -> Vec<Message> {
    println!("[DEBUG] Connected to Twitch, sending auth, nick, and join");
    vec![
        format!("PASS oauth:{}", user_token).to_ws_text(),
        format!("NICK {}", user_nick).to_ws_text(),
        format!("JOIN #{}", user_channel).to_ws_text(),
        "CAP REQ :twitch.tv/tags".to_ws_text(),
    ]
}
//     vec!(
//     format!("PASS oauth:{}", user_token).to_ws_text()),
//     format!("NICK {}", user_nick).to_ws_text()),
//     format!("JOIN #{}", user_channel).to_ws_text()),
//     "CAP REQ :twitch.tv/tags".to_ws_text())
// )
// }
