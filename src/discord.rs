use crate::{lotr, marvel_champions, CardSearch, DbCard};
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::Message,
    prelude::{TypeMap, TypeMapKey},
};
use std::str::FromStr;
use tracing::info;

pub struct MarvelChampionsCards;
impl TypeMapKey for MarvelChampionsCards {
    type Value = Vec<marvel_champions::Card>;
}

pub struct LOTRCards;
impl TypeMapKey for LOTRCards {
    type Value = Vec<lotr::Card>;
}

enum Game {
    LOTR,
    MarvelChampions,
}

impl FromStr for Game {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "lotr" => Ok(Self::LOTR),
            "marvel" => Ok(Self::MarvelChampions),
            _ => Err(()),
        }
    }
}

impl Game {
    fn search(
        card_db: &TypeMap,
        game: impl AsRef<str>,
        query: impl AsRef<str>,
    ) -> Option<Vec<CardDisplay>> {
        match Game::from_str(game.as_ref()) {
            Ok(Game::LOTR) => {
                let cards = card_db
                    .get::<LOTRCards>()
                    .expect("Expected LOTRCards in TypeMap");
                Some(
                    lotr::API::search(&cards, query.as_ref())
                        .into_iter()
                        .map(|card| card.into())
                        .collect(),
                )
            }
            Ok(Game::MarvelChampions) => {
                let cards = card_db
                    .get::<MarvelChampionsCards>()
                    .expect("Expected MarvelChampionsCards in TypeMap");
                Some(
                    marvel_champions::API::search(&cards, query.as_ref())
                        .into_iter()
                        .map(|card| card.into())
                        .collect(),
                )
            }
            _ => None,
        }
    }
}

/// A struct to hold the card data for display, so we don't need a Vec<Box<dyn DbCard>>
struct CardDisplay<'a> {
    pub name: &'a str,
    pub image_url: Option<String>,
}

impl<'a, T> From<&'a T> for CardDisplay<'a>
where
    T: DbCard,
{
    fn from(card: &'a T) -> Self {
        CardDisplay {
            name: card.name(),
            image_url: card.image_url(),
        }
    }
}

#[command]
#[min_args(2)]
#[usage = "<game> <query>"]
pub async fn card(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let game = args.single::<String>()?;
    let query = args.rest();
    let data = ctx.data.read().await;

    let cards = Game::search(&data, &game, &query);
    if let Some(cards) = cards {
        info!(
            "Cards found in '{}' for '{}' from '{}': {}",
            &game,
            &query,
            &msg.author.name,
            cards.len()
        );
        if cards.len() > 0 {
            for card in cards {
                if let Some(image) = card.image_url {
                    msg.channel_id
                        .send_message(&ctx.http, |m| {
                            m.embed(|e| {
                                e.image(image);

                                e
                            });

                            m
                        })
                        .await?;
                }
            }
        } else {
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("No cards found in game '{}' for '{}'.", &game, &query),
                )
                .await?;
        }
    } else {
        msg.channel_id
            .say(&ctx.http, "only valid games are: lotr, marvel")
            .await?;
    }

    Ok(())
}
