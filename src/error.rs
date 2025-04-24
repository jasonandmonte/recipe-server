extern crate serde_json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RecipeError {
    #[error("could not find recipes file: {0}")]
    RecipesNotFound(#[from] std::io::Error),
    #[error("could not read recipes file: {0}")]
    RecipesMisformat(#[from] serde_json::Error),
}
