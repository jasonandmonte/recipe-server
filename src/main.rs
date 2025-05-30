mod api;
mod error;
mod recipe;
mod templates;
mod web;

use error::*;
use recipe::*;
use templates::*;

extern crate fastrand;
extern crate log;
extern crate mime;

use axum::{
    self,
    extract::{Json, Path, Query, State},
    http,
    response::{self, IntoResponse},
    routing,
};
use clap::Parser;
use serde::Deserialize;
use sqlx::{Row, SqlitePool};
use std::sync::Arc;
use tokio::{net, sync::RwLock};
use tower_http::{services, trace};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Parser)]
struct Args {
    #[arg(short, long, name = "init-from")]
    init_from: Option<std::path::PathBuf>,
}

struct AppState {
    db: SqlitePool,
    current_recipe: Recipe,
}

// 404 Route handler
async fn handler_404(uri: http::Uri) -> axum::response::Response {
    log::error!("404 No route for {uri}");
    (http::StatusCode::NOT_FOUND, "404 Not Found").into_response()
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let db = SqlitePool::connect("sqlite://db/recipes.db").await?;
    sqlx::migrate!().run(&db).await?;

    if let Some(path) = args.init_from {
        let recipes = read_recipes(path)?;
        'next_recipe: for rr in recipes {
            let mut tx = db.begin().await?;
            let (r, tags) = rr.to_recipe();
            let recipe_insert = sqlx::query!(
                "INSERT INTO recipes (id, title, ingredients, instructions, recipe_source) VALUES ($1, $2, $3, $4, $5);",
                r.id,
                r.title,
                r.ingredients,
                r.instructions,
                r.recipe_source,
            )
            .execute(&mut *tx)
            .await;
            if let Err(e) = recipe_insert {
                eprintln!("error: recipe insert: {}: {}", r.id, e);
                tx.rollback().await?;
                continue;
            };
            for t in tags {
                let tag_insert = sqlx::query!(
                    "INSERT INTO tags (recipe_id, tag) VALUES ($1, $2);",
                    r.id,
                    t,
                )
                .execute(&mut *tx)
                .await;
                if let Err(e) = tag_insert {
                    eprintln!("error: tag insert: {} {}: {}", r.id, t, e);
                    tx.rollback().await?;
                    continue 'next_recipe;
                };
            }
            tx.commit().await?;
        }
        return Ok(());
    }

    let current_recipe = Recipe {
        id: "boil".to_string(),
        title: "Boil Water".to_string(),
        ingredients: "100 ml water".to_string(),
        instructions: "Add water to pot.\nHeat pot until water boils.".to_string(),
        recipe_source: "Jason Gonzales".to_string(),
    };
    let app_state = AppState { db, current_recipe };
    let state = Arc::new(RwLock::new(app_state));

    // RUST_LOG is the default env variable
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("recipe_server=debug,info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));

    let cors = tower_http::cors::CorsLayer::new()
        .allow_methods([http::Method::GET])
        .allow_origin(tower_http::cors::Any);

    let mime_favicon = "image/vnd.microsoft.icon".parse().unwrap();

    // API Routing
    let (api_router, api) = OpenApiRouter::with_openapi(api::ApiDoc::openapi())
        .nest("/api/v1", api::router())
        .split_for_parts();
    // API Docs
    let swagger_ui = SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone());
    let redoc_ui = Redoc::with_url("/redoc", api);
    let rapidoc_ui = RapiDoc::new("/api-docs/openapi.json").path("/rapidoc");

    // Website Routing
    let app = axum::Router::new()
        .route("/", routing::get(web::get_recipe))
        // NOTE: axum talks to tower-http
        .route_service(
            "/recipe.css",
            services::ServeFile::new_with_mime("assets/static/recipe.css", &mime::TEXT_CSS_UTF_8),
        )
        .route_service(
            "/favicon.ico",
            services::ServeFile::new_with_mime("assets/static/favicon.ico", &mime_favicon),
        )
        .merge(swagger_ui)
        .merge(redoc_ui)
        .merge(rapidoc_ui)
        .merge(api_router)
        .fallback(handler_404)
        .layer(cors)
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
