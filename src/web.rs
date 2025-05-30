use crate::*;

#[derive(Deserialize)]
pub struct GetRecipeParams {
    id: Option<String>,
    tags: Option<String>,
}

pub async fn get_recipe(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<GetRecipeParams>,
) -> Result<response::Response, http::StatusCode> {
    let mut app_state = app_state.write().await;
    let db = app_state.db.clone();

    // User has passed the id in the params
    if let GetRecipeParams { id: Some(id), .. } = params {
        let recipe_result = recipe::get(&db, &id).await;
        let result = match recipe_result {
            Ok((recipe, tags)) => {
                let tag_string = tags.join(", ");

                app_state.current_recipe = recipe.clone();
                let recipe = IndexTemplate::new(recipe.clone(), tag_string);
                Ok(response::Html(recipe.to_string()).into_response())
            }
            Err(e) => {
                log::warn!("recipe fetch failed: {}", e);
                Err(http::StatusCode::NOT_FOUND)
            }
        };
        return result;
    }

    // User passed tags in the params
    if let GetRecipeParams {
        tags: Some(tags), ..
    } = params
    {
        log::info!("recipe tags: {}", tags);
        let tags_vec: Vec<String> = tags.split(",").map(str::to_string).collect();
        let recipe_result = get_random_from_tags(&db, tags_vec).await;

        match recipe_result {
            Ok((recipe, _)) => {
                let uri = format!("/?id={}", recipe.id);
                return Ok(response::Redirect::to(&uri).into_response());
            }
            Err(sqlx::Error::RowNotFound) => {
                log::info!("tagged recipe selection was empty");
                return Err(http::StatusCode::NOT_FOUND);
            }
            Err(e) => {
                log::error!("tagged recipe selection database error: {}", e);
                panic!("tagged recipe selection database error");
            }
        }
    }

    // Default to a random joke
    let recipe_result = sqlx::query_scalar!("SELECT id FROM recipes ORDER BY RANDOM() LIMIT 1;")
        .fetch_one(&db)
        .await;

    match recipe_result {
        Ok(id) => {
            let uri = format!("/?id={}", id);
            Ok(response::Redirect::to(&uri).into_response())
        }
        Err(e) => {
            log::error!("recipe failed fetch one query: {}", e);
            panic!("recipe selection failed");
        }
    }
}
