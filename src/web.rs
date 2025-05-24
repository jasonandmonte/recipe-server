use crate::*;

#[derive(Deserialize)]
pub struct GetRecipeParams {
    id: Option<String>,
    tags: Option<String>,
}

/// Retrieves a random recipe ID that matches any of the passed tags.
async fn tagged_recipe(db: &SqlitePool, tags: &str) -> Result<Option<String>, sqlx::Error> {
    let mut tx = db.begin().await?;
    sqlx::query("DROP TABLE IF EXISTS qtags;")
        .execute(&mut *tx)
        .await?;
    sqlx::query("CREATE TEMPORARY TABLE qtags (tag VARCHR(200));")
        .execute(&mut *tx)
        .await?;
    for tag in tags.split(',') {
        sqlx::query("INSERT INTO qtags VALUES ($1);")
            .bind(tag)
            .execute(&mut *tx)
            .await?;
    }
    let recipe_ids = sqlx::query("SELECT DISTINCT recipe_id FROM tags JOIN qtags ON tags.tag = qtags.tag ORDER BY RANDOM() LIMIT 1;")
        .fetch_all(&mut *tx)
        .await?;
    let nrecipe_ids = recipe_ids.len();
    let result = if nrecipe_ids == 1 {
        Some(recipe_ids[0].get(0))
    } else {
        None
    };
    tx.commit().await?;

    Ok(result)
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

        let mut tags_string = String::new();
        for c in tags.chars() {
            if c.is_alphabetic() || c == ',' {
                let cl: String = c.to_lowercase().collect();
                tags_string.push_str(&cl);
            }
        }

        let recipe_result = tagged_recipe(&db, &tags_string).await;
        match recipe_result {
            Ok(Some(id)) => {
                let uri = format!("/?id={}", id);
                return Ok(response::Redirect::to(&uri).into_response());
            }
            Ok(None) => {
                log::info!("tagged recipe selection was empty");
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
