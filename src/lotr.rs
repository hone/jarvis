use crate::{CardSearch, DbCard};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use serde::Deserialize;

const CARDS_API: &str = "https://ringsdb.com/api/public/cards/";
const IMAGE_HOST: &str = "https://ringsdb.com";

#[derive(Deserialize)]
pub struct Card {
    pub code: String,
    pub name: String,
    pub text: Option<String>,
    pub imagesrc: Option<String>,
    pub pack_code: String,
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

    /// remove two player starter cards
    fn process_search(results: Vec<&Card>) -> Vec<&Card> {
        results
            .into_par_iter()
            .filter(|card| &card.pack_code != "Starter")
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cards_from_fixtures() -> Vec<Card> {
        serde_json::from_str(include_str!("../fixtures/ringsdb.json")).unwrap()
    }

    #[test]
    fn it_parses_all_cards() {
        let result = tokio_test::block_on(API::cards());

        assert!(result.is_ok());
    }

    #[test]
    fn it_searches() {
        let cards = cards_from_fixtures();

        let results: Vec<&Card> = API::search(&cards, "yazan");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn it_searches_removes_two_player_starter() {
        let cards = cards_from_fixtures();

        let results: Vec<&Card> = API::search(&cards, "arwen undomiel");
        assert_eq!(results.len(), 2);
    }
}
