use anyhow::Context;
use {
  anyhow::Result,
  serde::Deserialize
};

#[derive(Deserialize)]
pub struct Config {
  pub prefix: String,
  pub discord_token: String,
  pub chatgpt_key: String
}

impl Config {
  pub fn from_file() -> Result<Self> {
    toml::from_str(&std::fs::read_to_string("config.toml")?)
      .context("Failed to load ./config.toml")
  }
}