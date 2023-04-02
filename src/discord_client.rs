use {
  crate::config::Config,
  anyhow::{Context as ErrContext, Result},
  serenity::{
    prelude::*, async_trait, model::channel::Message,
    framework::standard::{
      StandardFramework, CommandResult, Args,
      macros::{command, group}
    }
  },
  chatgpt::prelude::*,
};

#[group]
#[commands(ping, prompt)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

struct ChatGPTArcPtr;
impl TypeMapKey for ChatGPTArcPtr {
  type Value = ChatGPT;
}

pub async fn init(config: &Config, gpt_client: ChatGPT) -> Result<()> {
  let framework = StandardFramework::new()
    .configure(|c| c.prefix(&config.prefix)) // set the bot's prefix
    .group(&GENERAL_GROUP);

  // Login with a bot token
  let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
  let mut client = Client::builder(&config.discord_token, intents)
    .event_handler(Handler)
    .framework(framework)
    .await
    .context("Error creating client")?;

  {
    let mut data = client.data.write().await;
    data.insert::<ChatGPTArcPtr>(gpt_client);
  }

  // start listening for events by starting a single shard
  client.start().await
    .context("An error occurred while running the client")?;

  Ok(())
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
  msg.reply(ctx, "Pong!").await?;
  Ok(())
}

#[command]
async fn prompt(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  //let prompt = "Describle why Rust is superior than C++"
  let prompt = args.rest();

  // Sending a message and getting the completion
  let data = ctx.data.read().await;
  let gpt_client = data.get::<ChatGPTArcPtr>().unwrap();
  let response = gpt_client.send_message(prompt).await?;
  
  msg.reply(ctx, &response.message().content).await?;
  Ok(())
}