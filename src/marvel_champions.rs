use crate::{CardSearch, DbCard};
use serde::Deserialize;

const IMAGE_HOST: &str = "https://marvelcdb.com";
const CARDS_API: &str = "https://marvelcdb.com/api/public/cards/";
const EDIT_DISTANCE: usize = 3;

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

struct Search;
impl CardSearch<Card> for Search {
    fn cards_api(&self) -> &str {
        CARDS_API
    }

    fn process_search<'a>(&self, results: Vec<&'a Box<Card>>) -> Vec<&'a Box<Card>> {
        results
            .into_iter()
            .filter(|card| card.duplicate_of_code.is_none())
            .collect()
    }
}

pub async fn cards() -> Result<Vec<Card>, reqwest::Error> {
    Ok(reqwest::get(CARDS_API).await?.json::<Vec<Card>>().await?)
}

pub fn search(cards: &Vec<Card>, query: impl AsRef<str>) -> Vec<&Card> {
    let exact_iter = cards.iter().filter(|card| {
        card.duplicate_of_code.is_none()
            && card.name.to_lowercase() == query.as_ref().to_lowercase()
    });
    let exact_matches: Vec<&Card> = exact_iter.clone().collect();

    if !exact_matches.is_empty() {
        exact_matches
    } else {
        cards
            .iter()
            .filter(|card| {
                card.duplicate_of_code.is_none()
                    && strsim::levenshtein(
                        &card.name.to_lowercase(),
                        &query.as_ref().to_lowercase(),
                    ) <= EDIT_DISTANCE
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cards_from_fixtures() -> Vec<Box<Card>> {
        serde_json::from_str(include_str!("../fixtures/marvelcdb.json")).unwrap()
    }

    #[test]
    fn it_parses_all_cards() {
        let search = Search {};
        let result = tokio_test::block_on(search.cards());
        assert!(result.is_ok());
    }

    #[test]
    fn it_searches_removing_dupes() {
        let cards = cards_from_fixtures();
        let search = Search {};

        let results: Vec<&Box<Card>> = search.search(&cards, "Enhanced Physique");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn it_searches_doesnt_care_baout_case() {
        let cards = cards_from_fixtures();
        let search = Search {};

        let results: Vec<&Box<Card>> = search.search(&cards, "enhanced physique");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn it_searches_for_dashed_names() {
        let cards = cards_from_fixtures();
        let search = Search {};

        let results: Vec<&Box<Card>> = search.search(&cards, "spider tracer");
        assert_eq!(results.len(), 1);
    }
}
