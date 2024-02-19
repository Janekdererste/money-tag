use crate::logging::init_logging;
use std::error::Error;

use crate::data_models::Record;
use crate::store::MongoDB;
use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::{get, post};
use axum::{Form, Router};
use clap::Parser;
use regex::Regex;
use serde::Deserialize;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::info;

mod data_models;
mod logging;
mod store;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct CommandLineArgs {
    #[arg(long, short)]
    user_db: String,
    #[arg(long, short)]
    secret_db: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = CommandLineArgs::parse();
    init_logging();
    let db = MongoDB::new(args.user_db.as_str(), args.secret_db.as_str()).await?;
    let app_state = AppState { db };

    info!("Configuring Router.");
    let router = Router::new()
        .route("/", get(index))
        .route("/create", get(create_record))
        .route("/handle-create", post(handle_create))
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);
    info!("Setting up Listener on 0.0.0.0:3000");
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("🚀 Starting Server!");
    axum::serve(listener, router).await.unwrap();

    Ok(())
}

async fn index<'a>(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.records("default").await {
        Ok(records) => (StatusCode::OK, IndexTemplate { records }).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("something went wrong the error was {e} "),
        )
            .into_response(),
    }
}

async fn create_record() -> CreateRecordTemplate {
    CreateRecordTemplate
}

async fn handle_create(
    State(state): State<AppState>,
    Form(data): Form<NewRecordForm>,
) -> impl IntoResponse {
    let re = Regex::new(r"[ ,;]").unwrap(); // Pattern matches space, comma, or semicolon
    let bla: Vec<String> = re
        .split(data.tag.as_str())
        .map(|part| part.to_owned())
        .collect();
    let new_record = Record {
        owner: String::from("default"),
        title: data.title.clone(),
        amount: data.amount,
        tags: bla,
    };

    match state.db.add_record(new_record).await {
        Ok(_) => Redirect::to("/").into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error adding record: {err}"),
        )
            .into_response(),
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    records: Vec<Record>,
}

#[derive(Template)]
#[template(path = "create-record.html")]
struct CreateRecordTemplate;

#[derive(Deserialize, Clone, Debug)]
struct NewRecordForm {
    title: String,
    amount: f32,
    tag: String,
}

#[derive(Debug, Clone)]
struct AppState {
    pub db: MongoDB,
}
