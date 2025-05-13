use crate::*;

use askama::Template;

#[derive(Template)]
#[template(path = "index.html")] // This directive links rust code/variables to html
pub struct IndexTemplate {
    recipe: Recipe,
    stylesheet: &'static str,
    tags: String,
}

impl IndexTemplate {
    pub fn new(recipe: Recipe, tags: String) -> Self {
        Self {
            recipe,
            stylesheet: "/recipe.css",
            tags,
        }
    }
}
