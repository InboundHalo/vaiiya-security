use std::error::Error;

use derive_getters::Getters;
use twilight_cache_inmemory::InMemoryCache;
use twilight_cache_inmemory::model::CachedMember;
use twilight_http::Client;
use twilight_model::guild::Permissions;
use twilight_model::id::Id;
use twilight_model::id::marker::{
    ApplicationMarker, ChannelMarker, GuildMarker, MessageMarker, UserMarker,
};
use twilight_model::user::User;
use twilight_util::permission_calculator::PermissionCalculator;

#[derive(Getters)]
pub struct Context {
    pub client: Client,
    pub cache: InMemoryCache,
    pub application_id: Id<ApplicationMarker>,
    pub bot: User,
}

impl Context {
    pub fn new(
        client: Client,
        cache: InMemoryCache,
        application_id: Id<ApplicationMarker>,
        bot: User,
    ) -> Self {
        Context {
            client,
            cache,
            application_id,
            bot,
        }
    }

    pub async fn get_last_bot_message(
        &self,
        channel_id: Id<ChannelMarker>,
        message_to_check: u16,
    ) -> Option<Id<MessageMarker>> {
        let messages = self
            .client
            .channel_messages(channel_id)
            .limit(message_to_check)
            .await
            .ok()?
            .model()
            .await
            .ok()?;

        for message in messages {
            if message.author.bot {
                return Some(message.id);
            }
        }

        None
    }

    pub async fn send_dm_to_user(
        &self,
        user_id: Id<UserMarker>,
        content: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let dm_channel = self
            .client
            .create_private_channel(user_id)
            .await?
            .model()
            .await?;

        self.client
            .create_message(dm_channel.id)
            .content(content)
            .await?;

        Ok(())
    }
}

pub trait GetAllGuildMembers {
    fn get_all_guild_members(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Vec<
        twilight_cache_inmemory::Reference<
            (Id<GuildMarker>, Id<twilight_model::id::marker::UserMarker>),
            CachedMember,
        >,
    >;
}

impl GetAllGuildMembers for InMemoryCache {
    fn get_all_guild_members(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Vec<
        twilight_cache_inmemory::Reference<
            (Id<GuildMarker>, Id<twilight_model::id::marker::UserMarker>),
            CachedMember,
        >,
    > {
        let Some(guild_member_ids) = self.guild_members(guild_id) else {
            return vec![];
        };

        guild_member_ids
            .iter()
            .flat_map(|user_marker| self.member(guild_id, *user_marker))
            .collect()
    }
}
