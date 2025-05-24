use crate::*;

pub async fn get_recipe(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(recipe_id): Path<String>,
) -> Result<response::Response, http::StatusCode> {
    let app = app_state.write().await;
    let db = &app.db;
    let recipe_result = recipe::get(db, &recipe_id).await;

    match recipe_result {
        Ok((recipe, tags)) => Ok(JSONRecipe::new(recipe, tags).into_response()),
        Err(e) => {
            log::warn!("api recipe fetch failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}
