use crate::logging::init_logging;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use askama::Template;
use axum::routing::get;
use axum::Router;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tracing::info;

mod logging;

#[tokio::main]
async fn main() {
    init_logging();

    let reload_counter = Arc::new(AtomicU32::new(0));
    let counter_ref = reload_counter.clone();

    info!("Configuring Router.");
    let router = Router::new()
        .route(
            "/",
            get(move || async {
                let counter_ref = counter_ref;
                let count = counter_ref.fetch_add(1, Ordering::Relaxed);
                index(count).await
            }),
        )
        .nest_service("/assets", ServeDir::new("assets"));
    info!("Setting up Listener on 0.0.0.0:3000");
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("🚀 Starting Server!");
    axum::serve(listener, router).await.unwrap();
}

async fn index(i: u32) -> IndexTemplate {
    info!("Reloaded {i} times");
    IndexTemplate { reloads: i }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    reloads: u32,
}
