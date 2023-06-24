extern crate dotenv;

use confy::ConfyError;
use confy::*;
use std::env;
use std::env::Args;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitchChannelInfo {
    pub channel: String,
    pub chat_bot_name: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitchConfig {
    pub user_auth_uri: String,
    pub oauth_uri: String,
    pub socket_server: String,
    pub secret: String,
    pub grant_type: String,
    pub client_id: String,
    pub response_type: String,
    pub redirect_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub twitch_config: TwitchConfig,
    pub twitch_channel_info: TwitchChannelInfo,
}

impl std::default::Default for Settings {
    fn default() -> Self {
        Self {
            twitch_config: TwitchConfig {
                user_auth_uri: "".to_string(),
                oauth_uri: "".to_string(),
                socket_server: "".to_string(),
                secret: "".to_string(),
                grant_type: "".to_string(),
                client_id: "".to_string(),
                response_type: "".to_string(),
                redirect_uri: "".to_string(),
            },
            twitch_channel_info: TwitchChannelInfo {
                channel: "".to_string(),
                chat_bot_name: "".to_string(),
                username: "".to_string(),
            },
        }
    }
}

impl Settings {
    pub fn new() -> Result<Self, ConfyError> {
        let env = dotenv::var("ENV").ok().unwrap();
        let mut env_path = ".env";
        dotenv::from_path(env_path).expect("Could not load env file");

        // config_path = "config.toml";
        let cfg: Settings = confy::load_path("config.toml")?;

        println!("settings {:?}", cfg);

        Ok(cfg)
    }
}
