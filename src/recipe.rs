
pub struct Recipe {
    pub title: &'static str,
    pub ingredients : &'static str,
    pub instructions: &'static str,
    pub source: &'static str,
}

pub const THE_RECIPE: Recipe = Recipe {
    title: "Example Recipe",
    ingredients: "ING1",
    instructions: "INST1",
    source: "http://www.example.com"
};
