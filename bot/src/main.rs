use common::bot::Bot;
use data::Database;
use embark_id_sync::EmbarkIDSync;
use std::sync::Arc;
use std::{env, io};
use tracing::{error, warn};
use tracing_appender::rolling;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Layer};
use twilight_model::id::Id;

#[tokio::main]
async fn main() {
    let _tracing_logger = register_tracing();

    let Ok(token) = env::var("DISCORD_TOKEN".to_string()) else {
        panic!("No DISCORD_TOKEN environment variables");
    };

    let mut bot = Bot::new(token);

    let database_path = match env::var("DATABASE_PATH".to_string()) {
        Ok(database_path) => database_path,
        Err(_) => {
            warn!("No DATABASE_PATH found in environment variables. Defaulting to database.db");
            "database.db".to_string()
        }
    };

    let database = Arc::new(Database::new(database_path).expect("Error with the database!"));

    let embark_id_sync = EmbarkIDSync::new(database).await;

    bot.register(embark_id_sync);

    bot.start_blocking().await;
}

pub fn register_tracing() -> tracing_appender::non_blocking::WorkerGuard {
    let file_appender = rolling::daily("logs", "log");
    let (non_blocking_writer, guard) = tracing_appender::non_blocking(file_appender);

    let stdout_layer = fmt::layer()
        .with_writer(io::stdout)
        .with_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()));

    let file_layer = fmt::layer()
        .with_writer(non_blocking_writer)
        .with_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()));

    let subscriber = tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer);

    tracing::subscriber::set_global_default(subscriber).expect("Unable to set a global subscriber");

    guard
}
