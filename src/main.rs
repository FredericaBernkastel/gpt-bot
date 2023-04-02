mod config;
mod discord_client;

use {
  chatgpt::prelude::*,
  anyhow::Result,
  config::Config,
};

#[tokio::main]
async fn main () -> Result<()> {
  // read config from ./config.toml
  let config = Config::from_file().unwrap();

  let gpt_client = ChatGPT::new(&config.chatgpt_key)?;

  discord_client::init(&config, gpt_client).await
}
