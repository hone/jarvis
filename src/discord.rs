use crate::{lotr, marvel_champions, CardSearch, DbCard};
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::Message,
    prelude::{SerenityError, TypeMapKey},
};

pub struct MarvelChampionsCards;
impl TypeMapKey for MarvelChampionsCards {
    type Value = Vec<marvel_champions::Card>;
}

pub struct LOTRCards;
impl TypeMapKey for LOTRCards {
    type Value = Vec<lotr::Card>;
}

#[command]
#[min_args(2)]
#[usage = "<game> <query>"]
pub async fn card(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let game = args.single::<String>()?;
    let query = args.rest();
    let data = ctx.data.read().await;

    match game.as_str() {
        "lotr" => {
            let cards = data
                .get::<LOTRCards>()
                .expect("Expected LOTRCards in TypeMap");
            display_card_images(&ctx, &msg, &lotr::API::search(&cards, &query)).await?
        }
        "marvel" => {
            let cards = data
                .get::<MarvelChampionsCards>()
                .expect("Expected LOTRCards in TypeMap");
            display_card_images(&ctx, &msg, &marvel_champions::API::search(&cards, &query)).await?
        }
        _ => {
            msg.channel_id
                .say(&ctx.http, "only valid games are: lotr, marvel")
                .await?;
        }
    };

    Ok(())
}

async fn display_card_images<T: DbCard>(
    ctx: &Context,
    msg: &Message,
    cards: &Vec<&T>,
) -> Result<(), SerenityError> {
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
                .await?;
        }
    }

    Ok(())
}
