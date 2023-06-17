extern crate confy;
extern crate dotenv;

use confy::*;
use confy::ConfyError;
use std::env;
use std::env::Args;
use std::path::{Path, PathBuf};

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitchConfig {
    pub uri: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub twitch_config: TwitchConfig
}

impl std::default::Default for Settings {
    fn default() -> Self { Self {
        twitch_config: TwitchConfig {
            uri: "".to_string()
        }
    }}

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