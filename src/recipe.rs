use crate::RecipeError;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct Recipe {
    pub title: String,
    pub ingredients: Vec<String>,
    pub instructions: Vec<String>,
    pub source: String,
}

pub fn read_recipes<P: AsRef<Path>>(recipes_path: P) -> Result<Vec<Recipe>, RecipeError> {
    let f = std::fs::File::open(recipes_path.as_ref())?;
    let recipes = serde_json::from_reader(f)?;
    Ok(recipes)
}
