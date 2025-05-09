use crate::RecipeError;
use serde::Deserialize;
use std::{collections::HashSet, ops::Deref, path::Path};

#[derive(Deserialize)]
pub struct JSONRecipe {
    pub id: String,
    pub title: String,
    pub ingredients: String,
    pub instructions: String,
    pub source: String,
    pub tags: HashSet<String>,
}

#[derive(Clone)]
pub struct Recipe {
    pub id: String,
    pub title: String,
    pub ingredients: String,
    pub instructions: String,
    pub recipe_source: String,
}

pub fn read_recipes<P: AsRef<Path>>(recipes_path: P) -> Result<Vec<JSONRecipe>, RecipeError> {
    let f = std::fs::File::open(recipes_path.as_ref())?;
    let recipes = serde_json::from_reader(f)?;
    Ok(recipes)
}

impl JSONRecipe {
    pub fn to_recipe(&self) -> (Recipe, impl Iterator<Item = &str>) {
        let recipe = Recipe {
            id: self.id.clone(),
            title: self.title.clone(),
            ingredients: self.ingredients.clone(),
            instructions: self.instructions.clone(),
            recipe_source: self.source.clone(),
        };

        let tags = self.tags.iter().map(String::deref);
        (recipe, tags)
    }
}
