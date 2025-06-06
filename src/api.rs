use crate::*;

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "recipe-server", description = "Recipe API")
    )
)]
pub struct ApiDoc;

pub fn router() -> OpenApiRouter<Arc<RwLock<AppState>>> {
    OpenApiRouter::new()
        .routes(routes!(get_recipe_by_id))
        .routes(routes!(get_random_recipe))
        .routes(routes!(get_recipe_by_tag))
        .routes(routes!(register))
        .routes(routes!(add_recipe))
}



#[utoipa::path(
    post,
    path = "/add-recipe",
    request_body(
        content = inline(JSONRecipe),
        description = "Add a recipe"
    ),
    responses(
        (status = 201, description = "Recipe was added", body = ()),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Auth error", body = authjwt::AuthError),
    )
)]
pub async fn add_recipe(
    _claims: authjwt::Claims,
    State(app_state): State<SharedAppState>,
    Json(recipe): Json<JSONRecipe>,
) -> axum::response::Response {
    let app_state = app_state.read().await;
    match recipe::add(&app_state.db, recipe).await {
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
        Ok(()) => StatusCode::CREATED.into_response(),
    }
}


#[utoipa::path(
    get,
    path = "/recipe/{recipe_id}",
    responses(
        (status = 200, description = "Get recipe by ID", body = [JSONRecipe]),
        (status = 404, description = "No matching recipe"),
    )
)]
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

#[utoipa::path(
    get,
    path = "/recipe/random",
    responses(
        (status = 200, description = "Get random recipe", body = [JSONRecipe]),
        (status = 404, description = "No recipes available"),
    )
)]
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

#[utoipa::path(
    get,
    path = "/recipe/by-tags",
    responses(
        (status = 200, description = "Get recipe that has at least one matching tag.", body = [JSONRecipe]),
        (status = 404, description = "No matching recipes"),
    )
)]
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

#[utoipa::path(
    post,
    path = "/register",
    request_body(
        content = inline(authjwt::Registration),
        description = "Get an API key",
    ),
    responses(
        (status = 200, description = "JSON Web Token", body = authjwt::AuthBody),
        (status = 401, description = "Registration failed", body = authjwt::AuthError),
    )
)]
pub async fn register(
    State(app_state): State<SharedAppState>,
    Json(registration): Json<authjwt::Registration>,
) -> axum::response::Response {
    let app_state = app_state.read().await;
    match authjwt::make_jwt_token(&app_state, &registration) {
        Err(e) => e.into_response(),
        Ok(token) => (StatusCode::OK, token).into_response(),
    }
}
