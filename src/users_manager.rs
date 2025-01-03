use anyhow::Result;
use msedge_tts::tts::SpeechConfig;
use std::{collections::HashMap, sync::LazyLock};
use tokio::sync::RwLock;

use serde::{Deserialize, Serialize};

use crate::{
    config_manager::ConfigManager,
    tts::{TTSVoice, TTSVoiceTemplate, TTS_VOICE_DATABASE, TTS_VOICE_TEMPLATE},
};

pub static USER_DB: LazyLock<RwLock<UserDatabase>> =
    LazyLock::new(|| RwLock::new(UserDatabase::load_config(UserDatabase::default()).unwrap()));

pub static BOT_VOICE: LazyLock<BotVoice> =
    LazyLock::new(|| BotVoice::load_config(BotVoice::default()).unwrap());

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotVoice {
    pub speech_config: SpeechConfig,
}
impl Default for BotVoice {
    fn default() -> Self {
        Self {
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
    pub fn new(nickname: impl Into<String>, speech_config: SpeechConfig) -> Self {
        Self {
            nickname: nickname.into(),
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
    pub fn get_speech_config(&mut self, nickname: &str) -> SpeechConfig {
        if let Some(speech_config) = self.users.get(nickname) {
            return speech_config.clone();
        } else {
            let speech_config = self.create_speech_config();
            self.add_user(nickname, speech_config.clone());
            return speech_config;
        }
    }

    pub fn create_speech_config(&self) -> SpeechConfig {
        self.filter_template(&TTS_VOICE_TEMPLATE).speech_config
    }

    pub fn add_user(&mut self, nickname: impl Into<String>, speech_config: SpeechConfig) {
        self.users.insert(nickname.into(), speech_config.clone());
        UserDatabase::save_config::<UserDatabase>(self).unwrap();
    }

    pub fn remove_user(&mut self, nickname: &str) {
        self.users.remove(nickname);
    }

    pub fn reset_user_voice(&mut self, nickname: &str) -> Result<SpeechConfig> {
        self.remove_user(nickname);
        let speech_config = self.create_speech_config();
        self.add_user(nickname, speech_config.clone());
        UserDatabase::save_config::<UserDatabase>(self)?;
        Ok(speech_config)
    }

    fn filter_template(&self, user_speech_template: &TTSVoiceTemplate) -> TTSVoice {
        let mut speech_config = TTS_VOICE_DATABASE
            .filter_locale(&user_speech_template.locale)
            .filter_gender(&user_speech_template.gender)
            .random();

        user_speech_template.pitch.and_then(|pitch| {
            speech_config.speech_config.pitch = pitch;
            Some(())
        });

        if let Some(pitch) = user_speech_template.pitch {
            speech_config.speech_config.pitch = pitch;
        }
        if let Some(rate) = user_speech_template.rate {
            speech_config.speech_config.rate = rate;
        }
        if let Some(volume) = user_speech_template.volume {
            speech_config.speech_config.volume = volume;
        }
        speech_config
    }
}
