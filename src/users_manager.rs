use std::{collections::HashMap, sync::LazyLock};
use tokio::sync::RwLock;

use msedge_tts::tts::SpeechConfig;
use serde::{Deserialize, Serialize};

use crate::{
    config_manager::ConfigManager,
    tts::{TTSGender, TTS_VOICE_DATABASE},
};

pub static USER_DB: LazyLock<RwLock<UserDatabase>> =
    LazyLock::new(|| RwLock::new(UserDatabase::load_config(UserDatabase::default()).unwrap()));

pub static BOT_VOICE: LazyLock<BotVoice> =
    LazyLock::new(|| BotVoice::load_config(BotVoice::default()).unwrap());

pub static USER_TEMPLATE_VOICES: LazyLock<UserSpeechTemplate> =
    LazyLock::new(|| UserSpeechTemplate::load_config(UserSpeechTemplate::default()).unwrap());

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
    pub fn get_speech_config(&mut self, nickname: &str) -> Option<SpeechConfig> {
        if let Some(speech_config) = self.users.get(nickname) {
            return Some(speech_config.clone());
        } else {
            let speech_config = self.create_speech_config(&USER_TEMPLATE_VOICES);
            self.add_user(nickname, speech_config.clone()?);
            UserDatabase::save_config::<UserDatabase>(self).unwrap();
            return speech_config;
        }
    }

    pub fn create_speech_config(
        &self,
        user_speech_template: &UserSpeechTemplate,
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
