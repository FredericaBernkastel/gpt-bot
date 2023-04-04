use {
  crate::config::Config,
  anyhow::{Context as ErrContext, Result},
  serenity::{
    prelude::*, async_trait,
    model::{
      gateway::Ready,
      application::{
        command::{Command, CommandOptionType},
        interaction::{Interaction, InteractionResponseType}
      }
    },
    framework::standard::{
      StandardFramework,
      macros::group
    }
  },
  chatgpt::prelude::*,
};

#[group]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
  async fn ready(&self, ctx: Context, _ready: Ready) {

    // setup slash commands
    Command::create_global_application_command(&ctx.http, |command|
      command.name("ping").description("Ping!")
    ).await.ok();

    Command::create_global_application_command(&ctx.http, |command|
      command.name("prompt").description("Issue a ChatGPT prompt").create_option(|option|
        option
          .name("message")
          .description("The prompt to send")
          .kind(CommandOptionType::String)
          .required(true)
      )
    ).await.ok();
  }

  // handle slash commands
  async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    if let Interaction::ApplicationCommand(command) = interaction {
      println!("Received command interaction: {:#?}", command);

      let content = match command.data.name.as_str() {
        "prompt" => {
          let prompt = command.data.options.get(0).unwrap()
            .value.as_ref().unwrap().to_string();

          command.defer(&ctx.http).await.ok();

          // Sending a message and getting the completion
          let data = ctx.data.read().await;
          let gpt_client = data.get::<_ChatGPI>().unwrap();
          let mut gpt_response = gpt_client
            .send_message(prompt).await
            .unwrap()
            .message()
            .content
            .to_string();
          gpt_response.truncate(2000);

          command.create_followup_message(&ctx.http, |response|
            response.content(gpt_response)
          ).await.ok();
          "".to_string()
        },
        _ => "not implemented :(".to_string(),
      };

      if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
          response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| message.content(content))
        })
        .await {
        println!("Cannot respond to slash command: {}", why);
      }
    }
  }
}

// boilerplate serenity forces us to write
struct _ChatGPI; impl TypeMapKey for _ChatGPI { type Value = ChatGPT; }

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

  client.data.write().await
    .insert::<_ChatGPI>(gpt_client);

  // start listening for events by starting a single shard
  client.start().await
    .context("An error occurred while running the client")?;

  Ok(())
}
