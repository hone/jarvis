use crate::{CardSearch, DbCard};
use serde::Deserialize;

const IMAGE_HOST: &str = "https://marvelcdb.com";
const CARDS_API: &str = "https://marvelcdb.com/api/public/cards/";

#[derive(Deserialize)]
pub struct Card {
    pub code: String,
    pub name: String,
    pub duplicate_of_code: Option<String>,
    pub duplicate_of_name: Option<String>,
    pub real_text: Option<String>,
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

impl Card {
    pub fn image_url(&self) -> Option<String> {
        self.imagesrc
            .as_ref()
            .map(|image| format!("{}{}", IMAGE_HOST, image))
    }
}

pub struct API;
impl CardSearch<Card> for API {
    fn cards_api() -> &'static str {
        CARDS_API
    }

    /// remove card duplicates
    fn process_search(results: Vec<&Card>) -> Vec<&Card> {
        results
            .into_iter()
            .filter(|card| card.duplicate_of_code.is_none())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cards_from_fixtures() -> Vec<Card> {
        serde_json::from_str(include_str!("../fixtures/marvelcdb.json")).unwrap()
    }

    #[test]
    fn it_parses_all_cards() {
        let result = tokio_test::block_on(API::cards());
        assert!(result.is_ok());
    }

    #[test]
    fn it_searches_removing_dupes() {
        let cards = cards_from_fixtures();

        let results: Vec<&Card> = API::search(&cards, "Enhanced Physique");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn it_searches_doesnt_care_baout_case() {
        let cards = cards_from_fixtures();

        let results: Vec<&Card> = API::search(&cards, "enhanced physique");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn it_searches_for_dashed_names() {
        let cards = cards_from_fixtures();

        let results: Vec<&Card> = API::search(&cards, "spider tracer");
        assert_eq!(results.len(), 1);
    }
}
