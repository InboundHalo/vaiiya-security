use std::env;
use std::sync::Arc;
use tracing::error;
use tracing::{debug, info};
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::{EventTypeFlags, Shard, StreamExt};
use twilight_http::Client;
use twilight_model::gateway::event::Event;
use twilight_model::gateway::{Intents, ShardId};
use twilight_model::guild::Permissions;
use twilight_model::id::Id;
use twilight_model::id::marker::ApplicationMarker;
use twilight_standby::Standby;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    info!("starting");

    let Ok(token) = env::var("DISCORD_TOKEN".to_string()) else {
        panic!("No DISCORD_TOKEN environment variable");
    };

    let application_id = match env::var("APPLICATION_ID".to_string()) {
        Err(_) => panic!("No APPLICATION_ID environment variable"),
        Ok(application_id) => match application_id.parse::<u64>() {
            Ok(application_id) => Id::new(application_id),
            Err(_) => panic!("Invalid APPLICATION_ID environment variable"),
        },
    };

    let client = Arc::new(Client::new(token.clone()));
    let cache = Arc::new(InMemoryCache::builder().build());
    let standby = Arc::new(Standby::new());

    let intents = Intents::GUILDS | Intents::GUILD_MESSAGES;
    let mut shard = Shard::new(ShardId::ONE, token, intents);

    debug!("registering commands");

    register_setup_command(Arc::clone(&client), application_id).await;

    debug!("listening to events");

    while let Some(event) = shard.next_event(EventTypeFlags::all()).await {
        let Ok(event) = event else {
            let receive_message_error = event.unwrap_err();
            error!("error receiving event {receive_message_error}");
            return;
        };

        handle_event(&event, Arc::clone(&client), Arc::clone(&cache)).await;
        cache.update(&event);
        standby.process(&event);
    }
}

async fn handle_event(event: &Event, client: Arc<Client>, cache: Arc<InMemoryCache>) {
    match event {
        Event::Ready(_) => {
            info!("ready");
        }
        _ => {}
    }
}

async fn register_setup_command(client: Arc<Client>, application_id: Id<ApplicationMarker>) {
    client
        .interaction(application_id)
        .create_global_command()
        .chat_input("setup", "Automatically sets up the bot")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .dm_permission(false)
        .await
        .expect("error registering setup command");

    info!("Registered /setup command globally");
}
