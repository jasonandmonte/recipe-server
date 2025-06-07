use leptos::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub title: String,
    pub ingredients: String,
    pub instructions: String,
    pub source: String,
    /// Tags start attached from the .json file
    pub tags: HashSet<String>,
}

pub async fn fetch(endpoint: String) -> Result<Recipe, Error> {
    use reqwasm::http::Request;

    let ep = format!(
        "http://localhost:3000/api/v1/{}",
        endpoint,
    );
    let result = Request::get(&ep)
        .send()
        .await?
        // convert it to JSON
        .json()
        .await?;
    Ok(result)
}