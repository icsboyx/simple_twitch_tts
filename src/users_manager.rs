use anyhow::Result;
use std::collections::HashMap;

use msedge_tts::tts::SpeechConfig;
use serde::{Deserialize, Serialize};

use crate::{
    config_manager::ConfigManager,
    irc_parser::IrcMessage,
    tts::{TTSGender, TTSMessage, TTS_VOICE_DATABASE},
    Args,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSpeechTemplate {
    pub locale: Option<String>,
    pub gender: Option<TTSGender>,
    pub pitch: Option<i32>,
    pub rate: Option<i32>,
    pub volume: Option<i32>,
}

impl Default for UserSpeechTemplate {
    fn default() -> Self {
        UserSpeechTemplate {
            locale: Some("it-IT".into()),
            gender: Some(TTSGender::Male),
            pitch: Some(0),
            rate: Some(30),
            volume: Some(0),
        }
    }
}

impl ConfigManager for UserSpeechTemplate {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotVoice {
    pub speech_config: SpeechConfig,
}
impl Default for BotVoice {
    fn default() -> Self {
        BotVoice {
            speech_config: SpeechConfig {
                voice_name: "Microsoft Server Speech Text to Speech Voice (it-IT, GiuseppeMultilingualNeural)".into(),
                audio_format: "audio-24khz-48kbitrate-mono-mp3".into(),
                pitch: 40,
                rate: 30,
                volume: 0,
            },
        }
    }
}

impl ConfigManager for BotVoice {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub nickname: String,
    pub speech_config: SpeechConfig,
}

impl User {
    pub fn new(nickname: String, speech_config: SpeechConfig) -> Self {
        User {
            nickname,
            speech_config,
        }
    }
}

impl ConfigManager for UserDatabase {}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserDatabase {
    pub users: HashMap<String, SpeechConfig>,
}

impl UserDatabase {
    pub fn get_speech_config(&self, nickname: &str) -> Option<SpeechConfig> {
        self.users.get(nickname).cloned()
    }

    pub fn create_speech_config(
        &self,
        user_speech_template: UserSpeechTemplate,
    ) -> Option<SpeechConfig> {
        let mut speech_config = TTS_VOICE_DATABASE
            .filter_locale(&user_speech_template.locale)
            .filter_gender(&user_speech_template.gender)
            .random();
        if user_speech_template.pitch.is_some() {
            speech_config.speech_config.pitch = user_speech_template.pitch.unwrap();
        }
        if user_speech_template.rate.is_some() {
            speech_config.speech_config.rate = user_speech_template.rate.unwrap();
        }
        if user_speech_template.volume.is_some() {
            speech_config.speech_config.volume = user_speech_template.volume.unwrap();
        }
        Some(speech_config.speech_config)
    }

    pub fn add_user(&mut self, nickname: impl Into<String>, speech_config: SpeechConfig) {
        self.users.insert(nickname.into(), speech_config);
    }
}

pub async fn start(args: Args) -> Result<()> {
    // Let get our communication queue
    let mut my_receiver = args.com_bus.subscribe::<IrcMessage>("USERS").await;

    // Load user database configuration or use default values and write to config file
    let mut user_db = UserDatabase::load_config::<UserDatabase>(UserDatabase::default())
        .await
        .unwrap();

    // Load user template voices configuration or use default values and write to config file
    let user_template_voices =
        UserSpeechTemplate::load_config::<UserSpeechTemplate>(UserSpeechTemplate::default())
            .await
            .unwrap();

    // Load bot voice configuration or use default values and write to config file
    let bot_voice = BotVoice::load_config::<BotVoice>(BotVoice::default())
        .await
        .unwrap();

    loop {
        tokio::select! {

            ret_val = my_receiver.recv() => {
                if ret_val.is_none() {
                    continue;
                }
                let ret_val = ret_val.unwrap();
                let user = ret_val.context.sender;
                let timestamp = ret_val.timestamp;
                let message = &ret_val.payload;

                let user_tts_speech_config = if let Some(user_speech_config) = user_db.get_speech_config(&user) {
                    user_speech_config
                } else {
                    if user == args.bot_info.get_name().await {
                        bot_voice.speech_config.clone()
                    } else {
                    let mut user_speech_config = user_db.create_speech_config(user_template_voices.clone()).unwrap();
                    user_db.add_user(user, user_speech_config.clone());
                    UserDatabase::save_config::<UserDatabase>(&user_db).await?;
                    if user_template_voices.pitch.is_some() {
                        user_speech_config.pitch = user_template_voices.pitch.unwrap();
                    }
                    if user_template_voices.rate.is_some() {
                        user_speech_config.rate = user_template_voices.rate.unwrap();
                    }
                    if user_template_voices.volume.is_some() {
                        user_speech_config.volume = user_template_voices.volume.unwrap();
                    }
                    user_speech_config}
                };




                args.com_bus.send("TTS", TTSMessage {
                    timestamp,
                    message: message.clone(),
                    user_speech_config: user_tts_speech_config,
                }).await?;
            }
        }
    }
    // Ok(())
}
