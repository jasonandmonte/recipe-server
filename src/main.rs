mod error;
mod recipe;
mod templates;

use error::*;
use recipe::*;
use templates::*;

extern crate fastrand;
extern crate log;
extern crate mime;

use axum::{
    self,
    extract::{Query, State},
    http,
    response::{self, IntoResponse},
    routing,
};
use clap::Parser;
use serde::Deserialize;
use sqlx::{Row, SqlitePool};
use std::sync::Arc;
use tokio::{net, sync::RwLock};
use tokio_stream::StreamExt;
use tower_http::{services, trace};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
struct Args {
    #[arg(short, long, name = "init-from")]
    init_from: Option<std::path::PathBuf>,
}

struct AppState {
    db: SqlitePool,
    current_recipe: Recipe,
}

#[derive(Deserialize)]
struct GetRecipeParams {
    id: Option<String>,
    tags: Option<String>,
}

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

async fn get_recipe(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<GetRecipeParams>,
) -> Result<response::Response, http::StatusCode> {
    let mut app_state = app_state.write().await;
    let db = app_state.db.clone();

    // User has passed the id in the params
    if let GetRecipeParams { id: Some(id), .. } = params {
        let recipe_result = sqlx::query_as!(Recipe, "SELECT * FROM recipes WHERE id = $1;", id)
            .fetch_one(&db)
            .await;
        let result = match recipe_result {
            Ok(recipe) => {
                let mut tags =
                    sqlx::query_scalar!("SELECT tag FROM tags WHERE recipe_id = $1;", recipe.id)
                        .fetch(&db);
                let mut tag_list: Vec<String> = Vec::new();
                while let Some(tag) = tags.next().await {
                    let tag = tag.unwrap_or_else(|e| {
                        log::error!("tag fetch failed: {}", e);
                        panic!("tag fetch failed")
                    });
                    tag_list.push(tag);
                }
                let tag_string = tag_list.join(", ");

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

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let db = SqlitePool::connect("sqlite://db/recipes.db").await?;
    sqlx::migrate!().run(&db).await?;

    // let recipes = read_recipes("assets/static/recipes.json")?;
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
