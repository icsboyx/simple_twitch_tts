#![allow(dead_code)]

use anyhow::Result;
use msedge_tts::{
    tts::{client::connect_async, SpeechConfig},
    voice::{get_voices_list, Voice},
};
use rand::Rng;
use serde::{Deserialize, Serialize};

use std::sync::LazyLock;

use crate::{
    audio_player::TTS_AUDIO_QUEUE,
    colors::Colorize,
    com::MSGQueue,
    commands::{BOT_COMMANDS, COMMAND_PREFIX},
    config_manager::ConfigManager,
    irc_parser::IrcMessage,
    twitch_client::TWITCH_MSG,
    users_manager::{BOT_VOICE, USER_DB},
    Args, WarningPrint,
};

pub static TTS_VOICE_DATABASE: LazyLock<TTSDatabase> = LazyLock::new(|| TTSDatabase::new());

pub static TTS_MSG_QUEUE: LazyLock<MSGQueue<String>> = LazyLock::new(|| MSGQueue::new());

static TRANSFORM_CHARS: &[(char, &str)] = &[('&', "and"), ('%', "percent")];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTSVoice {
    pub voice_config: Voice,
    pub speech_config: SpeechConfig,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TTSGender {
    Male,
    Female,
}

impl TTSGender {
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..=1) {
            0 => TTSGender::Male,
            _ => TTSGender::Female,
        }
    }
}

// Implement From<&str> for TTSGender
impl From<&str> for TTSGender {
    fn from(value: &str) -> Self {
        match value {
            "Male" => TTSGender::Male,
            "Female" => TTSGender::Female,
            _ => TTSGender::random(),
        }
    }
}

// Implement From<String> for TTSGender
impl From<String> for TTSGender {
    fn from(value: String) -> Self {
        Self::from(value.as_str()) // Reuse the &str implementation
    }
}

impl From<TTSGender> for String {
    fn from(value: TTSGender) -> Self {
        match value {
            TTSGender::Female => "Female".into(),
            TTSGender::Male => "Male".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTSDatabase {
    tts_configs: Vec<TTSVoice>,
}

impl TTSDatabase {
    pub fn new() -> Self {
        let voices = get_voices_list()
            .unwrap()
            .into_iter()
            .map(|voice| TTSVoice {
                speech_config: SpeechConfig::from(&voice),
                voice_config: voice,
            })
            .collect::<Vec<TTSVoice>>();
        TTSDatabase {
            tts_configs: voices,
        }
    }

    pub fn filter_gender(&self, gender: &Option<TTSGender>) -> TTSDatabase {
        if gender.is_none() {
            return self.clone();
        }
        let gender = gender.unwrap();

        let voices = self
            .tts_configs
            .iter()
            .filter(|voice| voice.voice_config.gender == Some(gender.into()))
            .map(|voice| voice.clone())
            .collect::<Vec<TTSVoice>>();

        if voices.is_empty() {
            WarningPrint!(
                "No voices found for gender: {}, please check you filter arguments. all filters are keysensitive",
                String::from(gender).orange()
            );
            return self.clone();
        }

        TTSDatabase {
            tts_configs: voices
                .iter()
                .map(|voice| voice.clone())
                .collect::<Vec<TTSVoice>>(),
        }
    }

    pub fn filter_locale(&self, locale: &Option<String>) -> TTSDatabase {
        if locale.is_none() {
            return self.clone();
        }
        let locale = locale.clone().unwrap();
        let voices = self
            .tts_configs
            .iter()
            .filter(|voice| voice.voice_config.locale == Some(locale.clone()))
            .map(|voice| voice.clone())
            .collect::<Vec<TTSVoice>>();

        if voices.is_empty() {
            WarningPrint!(
                "No voices found for locale: {}, please check you filter arguments. all filters are keysensitive",
                locale.orange()
            );
            return self.clone();
        }

        TTSDatabase {
            tts_configs: voices
                .iter()
                .map(|voice| voice.clone())
                .collect::<Vec<TTSVoice>>(),
        }
    }

    pub fn random(&self) -> TTSVoice {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.tts_configs.len());
        self.tts_configs[index].clone()
    }
}

impl ConfigManager for TTSDatabase {}

#[derive(Debug, Clone)]
pub struct TTSMessage {
    pub timestamp: i64,
    pub message: String,
    pub user_speech_config: SpeechConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TTSVoices {
    voices: Vec<Voice>,
}

impl TTSVoices {
    pub fn new() -> Self {
        TTSVoices {
            voices: get_voices_list().unwrap(),
        }
    }
}

impl ConfigManager for TTSVoices {}

pub static TTS_VOICE_TEMPLATE: LazyLock<TTSVoiceTemplate> =
    LazyLock::new(|| TTSVoiceTemplate::load_config(TTSVoiceTemplate::default()).unwrap());

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTSVoiceTemplate {
    pub locale: Option<String>,
    pub gender: Option<TTSGender>,
    pub pitch: Option<i32>,
    pub rate: Option<i32>,
    pub volume: Option<i32>,
}

impl Default for TTSVoiceTemplate {
    fn default() -> Self {
        TTSVoiceTemplate {
            locale: Some("it-IT".into()),
            gender: Some(TTSGender::Male),
            pitch: Some(-1),
            rate: Some(30),
            volume: Some(1),
        }
    }
}

impl ConfigManager for TTSVoiceTemplate {}

pub async fn start(_args: Args) -> Result<()> {
    let mut test_broadcast_rx = TWITCH_MSG.subscribe_broadcast().await;

    //
    BOT_COMMANDS
        .add_command(
            "list_voices".into(),
            Box::new(|irc_message| Box::pin(list_voices(irc_message))),
        )
        .await;

    loop {
        tokio::select! {

            Some(msg) = TTS_MSG_QUEUE.next() => {
                text_to_speech(&msg, &BOT_VOICE.speech_config).await?;
            }

            Ok(ret_val) = test_broadcast_rx.recv() => {
                match ret_val.context.command.as_str() {
                    "PRIVMSG" if !&ret_val.payload.starts_with(COMMAND_PREFIX) => {
                        let username = ret_val.context.sender;
                        let user_speech_config = USER_DB.write().await.get_speech_config(&username);
                        text_to_speech(&ret_val.payload, &user_speech_config).await?;
                    }
                    _ => {}
                };
            }
        }
    }

    pub async fn text_to_speech(text: &str, speech_config: &SpeechConfig) -> Result<()> {
        let text = text
            .chars()
            .map(|c| {
                TRANSFORM_CHARS
                    .iter()
                    .fold(c.to_string(), |acc, (char_to_replace, replacement)| {
                        acc.replace(*char_to_replace, replacement)
                    })
            })
            .collect::<String>();

        let mut tts = connect_async().await?;
        let audio = tts.synthesize(text.as_ref(), speech_config).await?;
        if audio.audio_bytes.is_empty() {
            return Ok(());
        }

        TTS_AUDIO_QUEUE.push_back(audio.audio_bytes).await;

        Ok(())
    }
}

pub async fn list_voices(_args: IrcMessage) -> Result<()> {
    let voices = TTS_VOICE_DATABASE.tts_configs.clone();
    TWITCH_MSG
        .send(format!(
            "Available voices: {}",
            voices
                .iter()
                .map(|voice| voice
                    .voice_config
                    .short_name
                    .clone()
                    .unwrap_or_else(|| "".into()))
                .collect::<Vec<_>>()
                .join(", ")
        ))
        .await?;
    Ok(())
}
