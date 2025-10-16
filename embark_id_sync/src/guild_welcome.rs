use data::GuildSettings;
use data::User;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::Client;
use twilight_model::application::command::Command;
use twilight_model::application::command::CommandType;
use twilight_model::application::interaction::application_command::CommandData;
use twilight_model::application::interaction::{InteractionData, InteractionType};
use twilight_model::channel::ChannelType;
use twilight_model::channel::message::component::{ActionRow, TextInput, TextInputStyle};
use twilight_model::channel::message::{Component, MessageFlags};
use twilight_model::channel::permission_overwrite::{PermissionOverwrite, PermissionOverwriteType};
use twilight_model::gateway::payload::incoming::GuildCreate;
use twilight_model::guild::Permissions;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::marker::InteractionMarker;
use twilight_standby::Standby;
use twilight_util::builder::command::CommandBuilder;
use twilight_util::builder::message::{ActionRowBuilder, ButtonBuilder};

use common::context;
use common::handler::Handler;
use data::Database;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use twilight_gateway::Event;

use crate::EmbarkIDSync;
use crate::context::Context;

impl EmbarkIDSync {
    pub async fn guild_event(&self, context: Arc<Context>, guild_create: &Box<GuildCreate>) {
        // Will send a message when someone just invites the bot to their sever

        // If we already know this guild that means the bot just started up so we can go ahead
        // and update the members
        if let Some(guild_settings) = self.database.get_guild_settings(&guild_create.id()) {
            // TODO: Update members
            return;
        }
        if let GuildCreate::Available(guild) = &**guild_create {
            debug!("Guild is Available");
            if let Some(time_joined) = guild.joined_at {
                let time_since_joined_server = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("How does this error?")
                    .as_secs()
                    - time_joined.as_secs() as u64;
                debug!("Time since joined is {:?}", time_since_joined_server); // Since I do not plan on restarting the bot that often lets go for a high
                // number
                let five_minutes = 60 * 5;
                if time_since_joined_server < five_minutes {
                    debug!("Less then 5 minutes!");
                    // Send join message
                    let channel_id_to_send_message_in = match guild.public_updates_channel_id {
                        Some(channel) => Some(channel),
                        None => match guild.system_channel_id {
                            Some(channel) => Some(channel),
                            None => match guild
                                .channels
                                .iter()
                                .find(|channel| channel.kind == ChannelType::GuildText)
                            {
                                Some(channel) => Some(channel.id),
                                None => None,
                            },
                        },
                    };

                    if let Some(channel_id) = channel_id_to_send_message_in {
                        debug!("sending setup message");
                        if let Err(error) = context
                            .client
                            .create_message(channel_id)
                            .content("Please run /setup to get started.")
                            .await
                        {
                            error!("Error sending first time message: {:?}", error);
                        };
                    }
                }
            }
        }
    }
}
