use crate::*;

pub async fn get_recipe_by_id(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(recipe_id): Path<String>,
) -> Result<response::Response, http::StatusCode> {
    let app = app_state.write().await;
    let db = &app.db;
    let recipe_result = recipe::get(db, &recipe_id).await;

    match recipe_result {
        Ok((recipe, tags)) => Ok(JSONRecipe::new(recipe, tags).into_response()),
        Err(e) => {
            log::warn!("api:get_recipe_by_id failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}

pub async fn get_random_recipe(
        State(app_state): State<Arc<RwLock<AppState>>>,
) -> Result<response::Response, http::StatusCode> {
    let app = app_state.write().await;
    let db = &app.db;
    let recipe_result = recipe::get_random(db).await;

    match recipe_result {
        Ok((recipe, tags)) => Ok(JSONRecipe::new(recipe, tags).into_response()),
        Err(e) => {
            log::warn!("api:get_random_recipe failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}

pub async fn get_recipe_by_tag(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Json(tags): Json<Vec<String>>,
) -> Result<response::Response, http::StatusCode> {
    log::info!("api:get_recipe_by_tag tags: {:?}", tags);
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    let recipe_result = recipe::get_random_from_tags(db, tags).await;
    
    match recipe_result {
        Ok((recipe, tags)) => Ok(JSONRecipe::new(recipe, tags).into_response()),
        Err(e) => {
            log::warn!("api:get_recipe_by_tag failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}
