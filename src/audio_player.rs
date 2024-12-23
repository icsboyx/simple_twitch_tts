use psimple::Simple;
use pulse::{
    sample::{Format, Spec},
    stream::Direction,
};
use std::{
    collections::VecDeque,
    io::Cursor,
    sync::{Arc, LazyLock},
};
use tokio::sync::{Notify, RwLock};

use crate::Args;
use anyhow::{Ok, Result};
use rodio::{Decoder, OutputStream};

#[derive(Debug, Clone)]
pub struct TTSQueue {
    queue: Arc<RwLock<VecDeque<Vec<u8>>>>,
    notify: Arc<tokio::sync::Notify>,
}

impl TTSQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn push_back(&self, audio: Vec<u8>) {
        self.queue.write().await.push_back(audio);
        println!("Audio buffer {:#?}", self.queue.read().await.len());
        self.notify.notify_waiters();
    }

    pub async fn next(&self) -> Option<Vec<u8>> {
        loop {
            println!("Audio buffer {:#?}", self.queue.read().await.len());
            if let Some(audio) = self.queue.write().await.pop_front() {
                return Some(audio);
            }
            self.notify.notified().await;
        }
    }

    pub async fn len(&self) -> usize {
        self.queue.read().await.len()
    }
}

pub static TTS_QUEUE: LazyLock<TTSQueue> = LazyLock::new(|| TTSQueue::new());

pub async fn start(_args: Args) -> Result<()> {
    while let Some(audio) = TTS_QUEUE.next().await {
        play_on_bot(audio).await?;
    }

    Ok(())
}

pub async fn play_audio(audio: Vec<u8>) -> Result<()> {
    use rodio::Decoder;
    use rodio::Sink;
    use std::io::Cursor;

    let cursor = Cursor::new(audio);
    let source = Decoder::new(cursor)?;

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}

// pub async fn play_windows(audio: Vec<u8>) -> Result<()> {
//     use rodio::Decoder;
//     use rodio::Sink;
//     use std::io::Cursor;

//     let cursor = Cursor::new(audio);
//     let source = Decoder::new(cursor)?;

//     let (_stream, stream_handle) = OutputStream::try_default().unwrap();
//     let sink = Sink::try_new(&stream_handle).unwrap();
//     sink.append(source);
//     sink.sleep_until_end();
//     Ok(())
// }

pub async fn play_on_bot(audio: Vec<u8>) -> Result<()> {
    let cursor = Cursor::new(audio);
    let source = Decoder::new(cursor)?;

    // let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let spec = Spec {
        format: Format::S16le,
        channels: 1,
        rate: 24000,
    };
    assert!(spec.is_valid());

    let sink = Simple::new(
        None,                // Use the default server
        "botox",             // Our application’s name
        Direction::Playback, // We want a playback stream
        Some("BOT.capture"), // Use the default device
        "botox tts",         // Description of our stream
        &spec,               // Our sample format
        None,                // Use default channel map
        None,                // Use default buffering attributes
    )
    .unwrap();

    let audio_data = source.into_iter().collect::<Vec<_>>();
    let audio = audio_data
        .iter()
        .flat_map(|&x| x.to_le_bytes().to_vec())
        .collect::<Vec<_>>();

    sink.write(&audio).unwrap();
    sink.drain().unwrap();

    Ok(())
}
