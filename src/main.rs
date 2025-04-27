mod error;
mod recipe;
mod templates;

use error::*;
use recipe::*;
use templates::*;

extern crate mime;
use axum::{self, extract::State, response, routing};
use std::sync::Arc;
use tokio::{net, sync::RwLock};
use tower_http::{services, trace};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

struct AppState {
    recipes: Vec<Recipe>,
}

async fn get_recipe(State(app_state): State<Arc<RwLock<AppState>>>) -> response::Html<String> {
    let app_state = app_state.read().await;
    // TODO: Take a random recipe from array
    // let nrecipes = app_state.recipes.len();
    let recipe = &app_state.recipes[0];
    let recipe = IndexTemplate::recipe(recipe);
    response::Html(recipe.to_string())
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let recipes = read_recipes("assets/static/recipes.json")?;
    let state = Arc::new(RwLock::new(AppState { recipes }));

    // RUST_LOG is the default env variable
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("recipe_server=debug,info"));
    
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));
    let mime_favicon = "image/vnd.microsoft.icon".parse().unwrap();
    
    let app = axum::Router::new()
        .route("/", routing::get(get_recipe))
        // NOTE: axum talks to tower-http
        .route_service(
            "/recipe.css",
            services::ServeFile::new_with_mime("assets/static/recipe.css", &mime::TEXT_CSS_UTF_8),
        )
        .route_service(
            "/favicon.ico",
            services::ServeFile::new_with_mime("assets/static/favicon.ico", &mime_favicon),
        )
        .layer(trace_layer)
        .with_state(state);

    let listener = net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = serve().await {
        eprintln!("recipes: error: {}", err);
        std::process::exit(1);
    }
}
