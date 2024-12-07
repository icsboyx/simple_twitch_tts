use crate::Args;
use anyhow::Result;
use rodio::OutputStream;

#[derive(Debug, Clone)]
pub struct TTSAudioMessage {
    pub timestamp: i64,
    pub audio: Vec<u8>,
}

pub async fn start(args: Args) -> Result<()> {
    println!("[DEBUG] Starting TTS Player");
    let mut my_receiver = args
        .com_bus
        .subscribe::<TTSAudioMessage>("TTS_PLAYER")
        .await;

    while let Some(msg) = my_receiver.recv().await {
        play_audio(msg.audio).await?;
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

pub async fn play_windows(audio: Vec<u8>) -> Result<()> {
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
