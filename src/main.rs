mod recipe;
mod templates;

use recipe::*;
use templates::*;

extern crate mime;
use axum::{self, response, routing};
use tokio::net;
use tower_http::services;

async fn get_recipe() -> response::Html<String> {
    // Reference to a constant
    let joke = IndexTemplate::recipe(&THE_RECIPE);
    response::Html(joke.to_string())
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let mime_favicon = "image/vnd.microsoft.icon".parse().unwrap();
    let app = axum::Router::new()
        .route("/", routing::get(get_recipe))
        // NOTE: axum talks to tower-http
        .route_service(
            "/recipe.css",
            services::ServeFile::new_with_mime(
                "assets/static/recipe.css",
                &mime::TEXT_CSS_UTF_8,
            ),
        )
        .route_service(
            "/favicon.ico",
            services::ServeFile::new_with_mime(
                "assets/static/favicon.ico",
                &mime_favicon,
            ),
        );
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