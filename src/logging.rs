use std::io;
use tracing::metadata::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

pub fn init_logging() {
    let stdout_layer = tracing_subscriber::fmt::Layer::new()
        .with_writer(io::stdout)
        .with_filter(LevelFilter::INFO);

    tracing_subscriber::registry().with(stdout_layer).init();
}
