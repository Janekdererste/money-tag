use crate::logging::init_logging;
use std::error::Error;

use crate::data_models::Record;
use crate::store::MongoDB;
use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::State;
use axum::http::StatusCode;
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
        .route("/create", post(create))
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);
    info!("Setting up Listener on 0.0.0.0:3000");
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("🚀 Starting to serve!");
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

async fn create(
    State(state): State<AppState>,
    Form(data): Form<NewRecordForm>,
) -> impl IntoResponse {
    // process the form. This should probably also have validation
    let regex = Regex::new(r"[,;]").unwrap(); // this doesn't have to be created each time i guess
    let tags: Vec<_> = regex
        .split(data.tag.as_str())
        .map(|part| part.to_owned())
        .collect();

    // create a db record and add it to the db.
    let new_record = Record {
        owner: "default".to_owned(), // this should be a user at some point
        title: data.title.to_owned(),
        amount: data.amount,
        tags,
    };

    match state.db.add_record(new_record).await {
        // return updated list in case of success
        Ok(_) => {
            let records = state.db.records("default").await.expect("Failed to load records from the database. Replace this with proper generic error handler");
            (StatusCode::OK, RecordList { records }).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {e}"),
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
#[template(path = "records.html")]
struct RecordList {
    records: Vec<Record>,
}

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
