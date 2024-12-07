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
    colors::Colorize, config_manager::ConfigManager, tts_player::TTSAudioMessage, Args,
    WarningPrint,
};

pub static TTS_VOICE_DATABASE: LazyLock<TTSDatabase> = LazyLock::new(|| TTSDatabase::new());

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
struct MSVoice {
    voices: Vec<Voice>,
}

impl MSVoice {
    pub fn new() -> Self {
        MSVoice {
            voices: get_voices_list().unwrap(),
        }
    }
}

impl ConfigManager for MSVoice {}

pub async fn start(args: Args) -> Result<()> {
    let _voices = MSVoice::load_config::<MSVoice>(MSVoice::new()).await?;
    println!("[DEBUG] Starting TTS Module");
    let mut my_receiver = args.com_bus.subscribe::<TTSMessage>("TTS").await;

    loop {
        tokio::select! {

                ret_val = my_receiver.recv() => {
                    if ret_val.is_none() {
                        continue;
                    }
                    let ret_val = ret_val.unwrap();
                    let mut tts = connect_async().await?;
                    let audio = tts.synthesize(&ret_val.message, &ret_val.user_speech_config).await?;
                    let tts_audio_message = TTSAudioMessage {
                        timestamp:  ret_val.timestamp,
                        audio: audio.audio_bytes,
                    };
                    drop(tts);
                    args.com_bus.send("TTS_PLAYER", tts_audio_message).await?;
                }
        }
    }
}
