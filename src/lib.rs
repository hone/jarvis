pub mod discord;
pub mod lotr;
pub mod marvel_champions;

use serde::de::DeserializeOwned;
use serenity::async_trait;

const EDIT_DISTANCE: usize = 3;

pub enum Card {
    LOTR(lotr::Card),
    MarvelChampions(marvel_champions::Card),
}

/// Trait for ThronesDB Card API
pub trait DbCard {
    fn name(&self) -> &str;
    fn image(&self) -> Option<&str>;
    fn image_host(&self) -> &str;

    fn image_url(&self) -> Option<String> {
        self.image()
            .as_ref()
            .map(|image| format!("{}{}", self.image_host(), image))
    }
}

#[async_trait]
/// Trait needed for providing fetching all cards and search for specific cards
pub trait CardSearch<T: DbCard + DeserializeOwned> {
    /// Card API URL
    fn cards_api() -> &'static str;

    /// process search results. By default, do nothing.
    fn process_search<'a>(results: Vec<&'a Box<T>>) -> Vec<&'a Box<T>> {
        results
    }

    /// fetch all cards from a given API
    async fn cards() -> Result<Vec<Box<T>>, reqwest::Error> {
        Ok(reqwest::get(Self::cards_api())
            .await?
            .json::<Vec<Box<T>>>()
            .await?)
    }

    /// search for cards that match the name. If no exact match is found will try to do a fuzzy
    /// search.
    fn search<'a>(cards: &'a Vec<Box<T>>, query: impl AsRef<str>) -> Vec<&'a Box<T>> {
        let exact_matches: Vec<&Box<T>> = cards
            .iter()
            .filter(|card| card.name().to_lowercase() == query.as_ref().to_lowercase())
            .collect();

        let matches = if !exact_matches.is_empty() {
            exact_matches
        // if can't find exact matches, try to fuzzy match
        } else {
            cards
                .iter()
                .filter(|card| {
                    strsim::levenshtein(&card.name().to_lowercase(), &query.as_ref().to_lowercase())
                        <= EDIT_DISTANCE
                })
                .collect()
        };

        Self::process_search(matches)
    }
}
