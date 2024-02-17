use crate::logging::init_logging;
use std::sync::{Arc, RwLock};

use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::State;
use axum::response::Redirect;
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::Deserialize;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::info;

mod logging;

#[tokio::main]
async fn main() {
    init_logging();

    info!("Configuring Router.");
    let router = Router::new()
        .route("/", get(index))
        .route("/create", get(create_record))
        .route("/handle-create", post(handle_create))
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState::default());
    info!("Setting up Listener on 0.0.0.0:3000");
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("🚀 Starting Server!");
    axum::serve(listener, router).await.unwrap();
}

async fn index<'a>(State(state): State<AppState>) -> IndexTemplate {
    IndexTemplate {
        records: state.records(),
    }
}

async fn create_record() -> CreateRecordTemplate {
    CreateRecordTemplate
}

async fn handle_create(
    State(mut state): State<AppState>,
    Form(data): Form<NewRecordForm>,
) -> impl IntoResponse {
    state.add_record(data);

    Redirect::to("/")
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    records: Vec<NewRecordForm>,
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

#[derive(Debug, Default, Clone)]
struct AppState {
    records: Arc<RwLock<Vec<NewRecordForm>>>,
}

impl AppState {
    fn records(&self) -> Vec<NewRecordForm> {
        self.records
            .read()
            .expect("Failed to acquire lock.")
            .clone()
    }

    fn add_record(&mut self, record: NewRecordForm) {
        self.records
            .write()
            .expect("Failed to aquire lock")
            .push(record);
    }
}
