use jarvis::{
    discord::{LOTRCards, MarvelChampionsCards, CARD_COMMAND},
    lotr, marvel_champions, CardSearch,
};
use serenity::client::Client;
use serenity::{
    async_trait,
    framework::standard::{
        macros::{group, hook},
        CommandError, StandardFramework,
    },
    model::{
        channel::Message,
        gateway::Ready,
        interactions::{
            ApplicationCommand, Interaction, InteractionData, InteractionResponseType,
            InteractionType,
        },
    },
    prelude::{Context, EventHandler},
};
use std::env;
use tracing::{error, info, instrument};

#[derive(Debug)]
struct SlashCommandHandler;

#[async_trait]
impl EventHandler for SlashCommandHandler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if interaction.kind == InteractionType::ApplicationCommand {
            if let Some(data) = interaction.data.as_ref() {
                match data {
                    InteractionData::ApplicationCommand(data) => {
                        let content: String = match data.name.as_str() {
                            "ping" => "Pong!".into(),
                            _ => "not implemented".into(),
                        };

                        if let Err(why) = interaction
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| message.content(content))
                            })
                            .await
                        {
                            error!("Cannot respond to slash command: {}", why);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    #[instrument(skip(ctx))]
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        if let Err(err) = ApplicationCommand::create_global_application_command(&ctx.http, |cmd| {
            cmd.name("ping").description("A simple ping command")
        })
        .await
        {
            error!("Could not create global application command: {:?}", err);
        }
        let interactions = ApplicationCommand::get_global_application_commands(&ctx.http).await;

        info!(
            "I have the following global slash command(s): {:?}",
            interactions
        );
    }
}

#[hook]
#[instrument]
// Currently, the instrument macro doesn't work with commands. Using the before hook instead.
async fn before(_ctx: &Context, _msg: &Message, command_name: &str) -> bool {
    true
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, cmd: &str, err: Result<(), CommandError>) {
    if let Err(why) = err {
        error!("Error in {}: {:?}", cmd, why);
    }
}

#[group]
#[commands(card)]
struct General;

#[tokio::main]
#[instrument]
async fn main() {
    #[cfg(debug_assertions)]
    dotenv::dotenv().ok();

    tracing_subscriber::fmt::init();

    let discord_token =
        env::var("DISCORD_TOKEN").expect("Please provide the env var DISCORD_TOKEN");
    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("application id is not a valid id");
    let marvel_champions_cards = marvel_champions::API::cards()
        .await
        .expect("Could net fetch cards for marvel champions");
    let lotr_cards = lotr::API::cards()
        .await
        .expect("Colud not fetch cards for lotr");

    let mut client = Client::builder(discord_token)
        .event_handler(SlashCommandHandler)
        .framework(
            StandardFramework::new()
                .configure(|c| c.prefix("!"))
                .before(before)
                .after(after)
                .group(&GENERAL_GROUP),
        )
        .application_id(application_id)
        .await
        .expect("Error creating client.");
    {
        let mut data = client.data.write().await;
        data.insert::<MarvelChampionsCards>(marvel_champions_cards);
        data.insert::<LOTRCards>(lotr_cards);
    }

    if let Err(why) = client.start().await {
        error!("an error occurred while running the client: {:?}", why);
    }
}
