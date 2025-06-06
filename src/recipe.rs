use crate::*;

use crate::RecipeError;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, ops::Deref, path::Path};

/// Represents a recipe as JSON object.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct JSONRecipe {
    pub id: String,
    pub title: String,
    pub ingredients: String,
    pub instructions: String,
    pub source: String,
    /// Tags start attached from the .json file
    pub tags: HashSet<String>,
}

/// Represents recipe stored in the database.
#[derive(Clone)]
pub struct Recipe {
    pub id: String,
    pub title: String,
    pub ingredients: String,
    pub instructions: String,
    pub recipe_source: String,
}

/// Reads JSON file and returns JSON recipes
pub fn read_recipes<P: AsRef<Path>>(recipes_path: P) -> Result<Vec<JSONRecipe>, RecipeError> {
    let f = std::fs::File::open(recipes_path.as_ref())?;
    let recipes = serde_json::from_reader(f)?;
    Ok(recipes)
}

/// Query db for recipe and tags with given ID.
pub async fn get(db: &SqlitePool, recipe_id: &str) -> Result<(Recipe, Vec<String>), sqlx::Error> {
    let recipe = sqlx::query_as!(Recipe, "SELECT * FROM recipes WHERE id = $1;", recipe_id)
        .fetch_one(db)
        .await?;

    type Tags = Vec<String>;
    let tags: Tags = sqlx::query_scalar!("SELECT tag FROM tags WHERE recipe_id = $1;", recipe_id)
        .fetch_all(db)
        .await?;

    Ok((recipe, tags))
}

/// Get random ID and then call get() to get recipe & tags.
pub async fn get_random(db: &SqlitePool) -> Result<(Recipe, Vec<String>), sqlx::Error> {
    let id = sqlx::query_scalar!("SELECT id FROM recipes ORDER BY RANDOM() LIMIT 1;")
        .fetch_one(db)
        .await?;

    get(db, &id).await
}

/// Get random recipe from given tags.
pub async fn get_random_from_tags(
    db: &SqlitePool,
    tags: Vec<String>,
) -> Result<(Recipe, Vec<String>), sqlx::Error> {
    let mut tx = db.begin().await?;
    sqlx::query("DROP TABLE IF EXISTS qtags;")
        .execute(&mut *tx)
        .await?;
    sqlx::query("CREATE TEMPORARY TABLE qtags (tag VARCHR(200));")
        .execute(&mut *tx)
        .await?;
    for tag in tags {
        sqlx::query("INSERT INTO qtags VALUES ($1);")
            .bind(tag)
            .execute(&mut *tx)
            .await?;
    }

    let row = sqlx::query("SELECT DISTINCT recipe_id FROM tags JOIN qtags ON tags.tag = qtags.tag ORDER BY RANDOM() LIMIT 1;")
        .fetch_optional(&mut *tx)
        .await?;

    tx.commit().await?;

    if let Some(row) = row {
        let id: String = row.get("recipe_id");
        get(db, &id).await
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}

/// Add recipe to recipes table and tags table in database.
pub async fn add(db: &SqlitePool, recipe: JSONRecipe) -> Result<(), sqlx::Error> {
    let mut jtx = db.begin().await?;

    sqlx::query!(
        r#"INSERT INTO recipes
        (id, title, ingredients, instructions, recipe_source)
        VALUES ($1, $2, $3, $4, $5);"#,
        recipe.id,
        recipe.title,
        recipe.ingredients,
        recipe.instructions,
        recipe.source,
    )
    .execute(&mut *jtx)
    .await?;

    for tag in recipe.tags {
        sqlx::query!(
            r#"INSERT INTO tags (recipe_id, tag) VALUES ($1, $2);"#,
            recipe.id,
            tag,
        )
            .execute(&mut *jtx)
            .await?;
    }

    jtx.commit().await?;
    Ok(())
}

impl JSONRecipe {
    pub fn new(recipe: Recipe, tags: Vec<String>) -> Self {
        let tags = tags.into_iter().collect();
        Self {
            id: recipe.id,
            title: recipe.title,
            ingredients: recipe.ingredients,
            instructions: recipe.instructions,
            source: recipe.recipe_source,
            tags,
        }
    }

    /// Convert from JSONRecipe to Recipe struct and tags iterator.
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

impl axum::response::IntoResponse for &JSONRecipe {
    fn into_response(self) -> axum::response::Response {
        (http::StatusCode::OK, axum::Json(&self)).into_response()
    }
}
