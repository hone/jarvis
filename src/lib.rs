pub mod discord;
pub mod lotr;
pub mod marvel_champions;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
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
pub trait CardSearch<T: DbCard + DeserializeOwned + Sync> {
    /// Card API URL
    fn cards_api() -> &'static str;

    /// process search results. By default, do nothing.
    fn process_search(results: Vec<&T>) -> Vec<&T> {
        results
    }

    /// fetch all cards from a given API
    async fn cards() -> Result<Vec<T>, reqwest::Error> {
        Ok(reqwest::get(Self::cards_api())
            .await?
            .json::<Vec<T>>()
            .await?)
    }

    /// search for cards that match the name. If no exact match is found will try to do a fuzzy
    /// search.
    fn search<'a>(cards: &'a Vec<T>, query: &str) -> Vec<&'a T> {
        let exact_matches: Vec<&T> = cards
            .par_iter()
            .filter(|card| card.name().to_lowercase() == query.to_lowercase())
            .collect();

        let matches = if !exact_matches.is_empty() {
            exact_matches
        // if can't find exact matches, try to fuzzy match
        } else {
            cards
                .par_iter()
                .filter(|card| {
                    strsim::levenshtein(&card.name().to_lowercase(), &query.to_lowercase())
                        <= EDIT_DISTANCE
                })
                .collect()
        };

        Self::process_search(matches)
    }
}
