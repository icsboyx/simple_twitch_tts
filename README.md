# Simple Twitch TTS

This project is a simple Text-to-Speech (TTS) application for Twitch streams, written in Rust.

## Features

- Convert chat messages to speech
- Easy integration with Twitch API
- Customizable voice settings

## Installation

1. Clone the repository:

```sh
git clone https://github.com/yourusername/simple_twitch_tts.git
cd simple_twitch_tts
```

2. Build the project:

```sh
cargo build --release
```

3. Run the application:

```sh
./target/release/simple_twitch_tts
```

## Usage

Config Files will be created on the first run of the application.
all configuration files will be created in `config/` directory.

2. Start the application:

```sh
./simple_twitch_tts
```

3. Config Files:

config/
├── BotVoice_config.toml
├── TwitchClient_config.toml
├── UserDatabase_config.toml
└── UserSpeechTemplate_config.toml
└── MSVoice_config.toml

---

- TwitchClient_config.toml:
  - This file contains the configuration for the Twitch Client.
  - You can change the client_id, client_secret, and the channel name.
  - All properties are required.
    - `client_id` and `client_secret` can be obtained by registering a new application on the [Twitch Developer Console]. See the [Twitch API Documentation] for more information.

```toml
    file_name = "twitch_client_config.toml"
    server_address = "wss://irc-ws.chat.twitch.tv:443"
    nick = "justinfan123"
    token = "oauth:1234567890"
    channel = "icsboyx"
    log_level = "info"
    anti_idle = 180
```

---

- BotVoice_config.toml:
  - This file contains the configuration for the bot voice.
  - You can change the voice, rate, pitch, volume, and language of the bot voice.
  - All available voices can be found on the [Microsoft Text-to-Speech API] or inside the `config/MSVoice_config.toml` file.

```toml
    [speech_config]
    voice_name = "Microsoft Server Speech Text to Speech Voice (it-IT, GiuseppeMultilingualNeural)"
    audio_format = "audio-24khz-48kbitrate-mono-mp3"
    pitch = 40
    rate = 30
    volume = 0
```

---

- UserDatabase_config.toml:
  - This file contains the configuration for the User Database, of assigned Voices(SpeechConfigs).
  - You can change the voice for a specific user.
  - The default voice is constructed from `config/UserSpeechTemplate_config.toml` file.

---

- UserSpeechTemplate_config.toml:
  - This file contains the configuration for the User Speech Template.
  - You can change the template for the user speech.
    - Commenting out `locale`, `gender`, randomizes the voice.
    - You can also change the `pitch`, `rate`, and `volume` for the user speech.

```toml
    locale = "it-IT"
    gender = "Male"
    pitch = 0
    rate = 30
    volume = 0
```

---

- BotVoice_config.toml:
  - This file contains the configuration for the bot voice.
  - You can change the voice_name choosing from the available voices in the `config/MSVoice_config.toml` file.
  - you can also change the `pitch`, `rate`, and `volume` for the bot speech.(audio_format .... Boh :P)

```toml
    [speech_config]
    voice_name = "Microsoft Server Speech Text to Speech Voice (it-IT, GiuseppeMultilingualNeural)"
    audio_format = "audio-24khz-48kbitrate-mono-mp3"
    pitch = 40
    rate = 30
    volume = 0
```

---

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
