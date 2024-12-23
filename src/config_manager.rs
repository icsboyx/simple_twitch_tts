#![allow(dead_code)]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{any::type_name, fmt::Debug, io::Write};

use crate::ErrorPrint;

static CONFIG_FOLDER_NAME: &str = "config";
static FILE_SUFFIX: &str = "_config";
static FILE_EXTENSION: &str = ".toml";

pub trait ConfigManager {
    fn load_config<T>(default_config: impl Serialize + Debug) -> Result<T>
    where
        T: for<'a> Deserialize<'a> + Serialize + Default + Debug,
    {
        // check if twitch toml file exists
        // if not, use anonymous default config for twitch

        let config_file_name = type_name::<T>().rsplit("::").next().unwrap().to_string()
            + FILE_SUFFIX
            + FILE_EXTENSION;

        let config_file_name = std::path::Path::new(CONFIG_FOLDER_NAME).join(&config_file_name);
        println!(
            "[DEBUG] Loading config from {}",
            config_file_name.to_string_lossy()
        );
        let config_content = std::fs::read_to_string(&config_file_name);
        match config_content {
            Ok(config) => match toml::from_str(&config) {
                Ok(config) => {
                    return Ok(config);
                }
                Err(err) => {
                    ErrorPrint!(
                        "Failed to parse config file: {}. Using default config",
                        err.message()
                    );
                    ErrorPrint!("Please check the config file: {}, delete it and restart the program, or fix the error in the file", config_file_name.to_string_lossy());
                    return Ok(serde_json::from_value(serde_json::to_value(
                        default_config,
                    )?)?);
                }
            },
            Err(_) => {
                ErrorPrint!(
                    "No {} config file found. Using default config. Saving default config to it",
                    config_file_name.to_string_lossy()
                );
                {
                    Self::save_config::<T>(&default_config)?;
                }
                Ok(serde_json::from_value(serde_json::to_value(
                    default_config,
                )?)?)
            }
        }
    }
    fn save_config<T>(config: impl Serialize) -> Result<()> {
        let config_toml = toml::to_string(&config)?;

        let config_file_name = type_name::<T>().rsplit("::").next().unwrap().to_string()
            + FILE_SUFFIX
            + FILE_EXTENSION;

        let config_file_name = std::path::Path::new(CONFIG_FOLDER_NAME).join(&config_file_name);

        if std::path::Path::new(CONFIG_FOLDER_NAME).exists() == false {
            std::fs::create_dir(CONFIG_FOLDER_NAME)?;
        }
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(config_file_name)?
            .write_all(config_toml.as_bytes())?;
        Ok(())
    }
    fn generate<T>(config: impl Serialize) -> Result<()> {
        let config_toml = toml::to_string(&config)?;

        let config_file_name = type_name::<T>().rsplit("::").next().unwrap().to_string()
            + FILE_SUFFIX
            + FILE_EXTENSION;

        let config_file_name = std::path::Path::new(CONFIG_FOLDER_NAME).join(&config_file_name);

        if std::path::Path::new(CONFIG_FOLDER_NAME).exists() == false {
            std::fs::create_dir(CONFIG_FOLDER_NAME)?;
        }
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(config_file_name)?
            .write_all(config_toml.as_bytes())?;
        Ok(())
    }
}

pub fn filename(file_name: &str) -> String {
    format!("{}{}{}", file_name, FILE_SUFFIX, FILE_EXTENSION)
}
