use std::env;
use std::sync::Arc;
use tracing::{Level, debug, info};
use tracing::{error, warn};
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::{EventTypeFlags, Shard, StreamExt};
use twilight_http::Client;
use twilight_model::gateway::{Intents, ShardId};
use twilight_model::id::Id;
use twilight_model::id::marker::ApplicationMarker;
use twilight_standby::Standby;

use crate::data::Database;

mod data;
mod event_handler;

pub struct Context {
    client: Client,
    cache: InMemoryCache,
    application_id: Id<ApplicationMarker>,
    database: Database,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .pretty()
        .with_line_number(true)
        .with_max_level(Level::DEBUG)
        .init();

    info!("starting");

    let Ok(token) = env::var("DISCORD_TOKEN".to_string()) else {
        panic!("No DISCORD_TOKEN environment variables");
    };

    let application_id = match env::var("APPLICATION_ID".to_string()) {
        Err(_) => panic!("No APPLICATION_ID environment variables"),
        Ok(application_id) => match application_id.parse::<u64>() {
            Ok(application_id) => Id::new(application_id),
            Err(_) => panic!("Invalid APPLICATION_ID environment variable"),
        },
    };

    let database_path = match env::var("DATABASE_PATH".to_string()) {
        Ok(database_path) => database_path,
        Err(_) => {
            warn!("No DATABASE_PATH found in environment variables. Defaulting to database.db");
            "database.db".to_string()
        }
    };

    let database = Database::new(database_path).expect("Error with the database!");

    let context = Arc::new(Context {
        client: Client::new(token.clone()),
        cache: InMemoryCache::builder().build(),
        application_id,
        database,
    });

    let standby = Arc::new(Standby::new());

    let intents = Intents::all();
    let mut shard = Shard::new(ShardId::ONE, token, intents);

    debug!("registering commands");

    event_handler::register_setup_command(Arc::clone(&context)).await;

    debug!("listening to events");

    while let Some(event) = shard.next_event(EventTypeFlags::all()).await {
        let Ok(event) = event else {
            let receive_message_error = event.unwrap_err();
            error!("error receiving event {receive_message_error}");
            return;
        };

        event_handler::handle_event(&event, Arc::clone(&context)).await;
        context.cache.update(&event);
        standby.process(&event);
    }
}
