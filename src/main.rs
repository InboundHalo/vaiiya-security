use std::env;
use std::sync::Arc;
use tracing::error;
use tracing::{debug, info};
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::{EventTypeFlags, Shard, StreamExt};
use twilight_http::Client;
use twilight_model::application::interaction::{InteractionData, InteractionType};
use twilight_model::channel::ChannelType;
use twilight_model::channel::permission_overwrite::{PermissionOverwrite, PermissionOverwriteType};
use twilight_model::gateway::event::Event;
use twilight_model::gateway::{Intents, ShardId};
use twilight_model::guild::Permissions;
use twilight_model::id::Id;
use twilight_model::id::marker::{ApplicationMarker, GuildMarker};
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

    let intents = Intents::all();
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
    info!("{event:#?}");

    match event {
        Event::Ready(_) => {
            info!("ready");
        },
        Event::InteractionCreate(interaction) => {
            match interaction.kind {
                InteractionType::Ping => {}
                InteractionType::ApplicationCommand => {
                    let Some(data) = &interaction.data else { return; };
                    let InteractionData::ApplicationCommand(command) = data else { return; };

                    
                    if command.name == "setup" {
                        // TODO: respond to the user
                        let Some(guild_id) = &interaction.guild_id else { error!("Guild not found");
                        return;
                        };
                        info!("guild exists {}", guild_id);
                        match setup_verification(client, guild_id.clone()).await {
                            Ok(_) => {
                                info!("setup complete");
                            }
                            Err(e) => {
                                match e {
                                    SetupErrors::CouldNotCreateChannel => {
                                        info!("couldn't create channel");
                                    }
                                    SetupErrors::CouldNotSendMessage => {
                                        info!("couldn't send message");
                                    }
                                    SetupErrors::CouldNotCreateRole => {
                                        info!("couldn't create role");
                                    }
                                }
                            }
                        }
                    }
                }
                InteractionType::MessageComponent => {}
                InteractionType::ApplicationCommandAutocomplete => {}
                InteractionType::ModalSubmit => {}
                _ => error!("unknown interaction type"),
            }
        }
        _ => {}
    }
}

enum SetupErrors {
    CouldNotCreateChannel,
    CouldNotSendMessage,
    CouldNotCreateRole,
}

async fn setup_verification(client: Arc<Client>, guild_id: Id<GuildMarker>) -> Result<(), SetupErrors> {
    let everyone_role_id = Id::new(guild_id.get());

    let channel = client
        .create_guild_channel(guild_id, "verify")
        .kind(ChannelType::GuildText)
        .permission_overwrites(&[
            PermissionOverwrite {
                allow: Permissions::VIEW_CHANNEL,
                deny: Permissions::SEND_MESSAGES | Permissions::ADD_REACTIONS,
                id: everyone_role_id,
                kind: PermissionOverwriteType::Role,
            }
        ])
        .await
        .map_err(|_| SetupErrors::CouldNotCreateChannel)?
        .model()
        .await
        .map_err(|_| SetupErrors::CouldNotCreateChannel)?;
    
    info!("Created channel: {} ({})", channel.name.unwrap_or("Unknown".to_string()), channel.id);

    // TODO: Make a button that opens a modal
    let message = client
        .create_message(channel.id)
        .content("verification message")
        .await.map_err(|_| SetupErrors::CouldNotSendMessage)?
        .model()
        .await.map_err(|_| SetupErrors::CouldNotSendMessage)?;

    info!("sent message: {} in channel {}", message.content, channel.id);

    // TODO: SAVE ROLE ID TO DB
    let role = client.create_role(guild_id)
        .name("verified")
        .color(6291322) // Just a green color I got from the color picker #00c822 TODO: change this maybe?
        .mentionable(false)
        .permissions(Permissions::empty())
        .await.map_err(|_| SetupErrors::CouldNotCreateRole)?
        .model()
        .await.map_err(|_| SetupErrors::CouldNotCreateRole)?;

    info!("created role: {} ({})", role.name, role.id);

    Ok(())
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
