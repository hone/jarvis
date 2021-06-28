use crate::{CardSearch, DbCard};
use serde::Deserialize;

const CARDS_API: &str = "https://ringsdb.com/api/public/cards/";
const IMAGE_HOST: &str = "https://ringsdb.com";

#[derive(Deserialize)]
pub struct Card {
    pub code: String,
    pub name: String,
    pub text: Option<String>,
    pub imagesrc: Option<String>,
}

impl DbCard for Card {
    fn name(&self) -> &str {
        &self.name
    }

    fn image(&self) -> Option<&str> {
        self.imagesrc.as_ref().map(|i| i.as_str())
    }

    fn image_host(&self) -> &str {
        IMAGE_HOST
    }
}

pub struct API;
impl CardSearch<Card> for API {
    fn cards_api() -> &'static str {
        CARDS_API
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cards_from_fixtures() -> Vec<Box<Card>> {
        serde_json::from_str(include_str!("../fixtures/ringsdb.json")).unwrap()
    }

    #[test]
    fn it_parses_all_cards() {
        let result = tokio_test::block_on(Search::cards());

        assert!(result.is_ok());
    }

    #[test]
    fn it_searches() {
        let search = Search {};
        let cards = cards_from_fixtures();

        let results: Vec<&Box<Card>> = API::search(&cards, "yazan");
        assert_eq!(results.len(), 1);
    }
}
