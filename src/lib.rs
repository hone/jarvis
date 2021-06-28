pub mod lotr;
pub mod marvel_champions;

use serde::de::DeserializeOwned;
use serenity::async_trait;

const EDIT_DISTANCE: usize = 3;

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
pub trait CardSearch<T: DbCard + DeserializeOwned> {
    fn cards_api(&self) -> &str;

    fn process_search<'a>(&self, results: Vec<&'a Box<T>>) -> Vec<&'a Box<T>> {
        results
    }

    async fn cards(&self) -> Result<Vec<Box<T>>, reqwest::Error> {
        Ok(reqwest::get(self.cards_api())
            .await?
            .json::<Vec<Box<T>>>()
            .await?)
    }

    fn search<'a>(&self, cards: &'a Vec<Box<T>>, query: impl AsRef<str>) -> Vec<&'a Box<T>> {
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

        self.process_search(matches)
    }
}
