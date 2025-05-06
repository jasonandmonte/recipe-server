use crate::*;

use askama::Template;

#[derive(Template)]
#[template(path = "index.html")] // This directive links rust code/variables to html
pub struct IndexTemplate {
    recipe: Recipe,
    stylesheet: &'static str,
}

impl IndexTemplate {
    pub fn recipe(recipe: Recipe) -> Self {
        Self {
            recipe,
            stylesheet: "/recipe.css",
        }
    }
}
