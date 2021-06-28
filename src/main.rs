use jarvis::marvel_champions;
use serenity::client::Client;
use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        interactions::{
            ApplicationCommand, Interaction, InteractionData, InteractionResponseType,
            InteractionType,
        },
        prelude::Message,
    },
    prelude::{Context, EventHandler, TypeMapKey},
};
use std::env;

struct MarvelChampionsCards;
impl TypeMapKey for MarvelChampionsCards {
    type Value = Vec<marvel_champions::Card>;
}

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
                            println!("Cannot respond to slash command: {}", why);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let data = ctx.data.read().await;
        let cards = data
            .get::<MarvelChampionsCards>()
            .expect("Expected MarvelChampionsCards in TypeMap");

        let parts = msg.content.split_whitespace().collect::<Vec<&str>>();
        if parts.len() > 0 {
            let command = parts[0];
            let args = &parts[1..];
            if command == "!card" {
                let query = args.join(" ");
                let cards = marvel_champions::search(&cards, &query);
                for card in cards {
                    if let Some(image) = card.image_url() {
                        msg.channel_id
                            .send_message(&ctx.http, |m| {
                                m.embed(|e| {
                                    e.image(image);

                                    e
                                });

                                m
                            })
                            .await
                            .unwrap();
                    }
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        ApplicationCommand::create_global_application_command(&ctx.http, |cmd| {
            cmd.name("ping").description("A simple ping command")
        })
        .await
        .unwrap();
        let interactions = ApplicationCommand::get_global_application_commands(&ctx.http).await;

        println!(
            "I have the following global slash command(s): {:?}",
            interactions
        );
    }
}

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    dotenv::dotenv().ok();

    let discord_token =
        env::var("DISCORD_TOKEN").expect("Please provide the env var DISCORD_TOKEN");
    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("application id is not a valid id");
    let marvel_champions_cards = marvel_champions::cards()
        .await
        .expect("Could net fetch cards from marvel champions");

    let mut client = Client::builder(discord_token)
        .event_handler(SlashCommandHandler)
        .application_id(application_id)
        .await
        .expect("Error creating client.");
    {
        let mut data = client.data.write().await;
        data.insert::<MarvelChampionsCards>(marvel_champions_cards);
    }

    if let Err(why) = client.start().await {
        println!("an error occurred while running the client: {:?}", why);
    }
}
