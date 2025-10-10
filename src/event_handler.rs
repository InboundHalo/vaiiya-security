use std::sync::Arc;
use tracing::debug;
use tracing::error;
use tracing::info;
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::Client;
use twilight_model::application::interaction::{InteractionData, InteractionType};
use twilight_model::channel::ChannelType;
use twilight_model::channel::message::component::{ActionRow, TextInput, TextInputStyle};
use twilight_model::channel::message::{Component, MessageFlags};
use twilight_model::channel::permission_overwrite::{PermissionOverwrite, PermissionOverwriteType};
use twilight_model::gateway::event::Event;
use twilight_model::guild;
use twilight_model::guild::Permissions;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::Id;
use twilight_model::id::marker::UserMarker;
use twilight_model::id::marker::{ChannelMarker, GuildMarker, MessageMarker, RoleMarker};
use twilight_util::builder::message::{ActionRowBuilder, ButtonBuilder};

use crate::Context;
use crate::data::EmbarkID;
use crate::data::GuildSettings;
use crate::data::User;

pub async fn handle_event(event: &Event, context: Arc<Context>) {
    match event {
        Event::Ready(_) => {
            info!("ready");
        }
        Event::InteractionCreate(interaction) => {
            let Some(data) = &interaction.data else {
                return;
            };

            match interaction.kind {
                InteractionType::Ping => {}
                InteractionType::ApplicationCommand => {
                    let InteractionData::ApplicationCommand(command) = data else {
                        return;
                    };

                    if command.name == "setup" {
                        let Some(guild_id) = &interaction.guild_id else {
                            error!("Guild not found");
                            return;
                        };
                        info!("guild exists {}", guild_id);
                        match setup_verification(context, guild_id.clone()).await {
                            Ok(_) => {
                                info!("setup complete");
                            }
                            Err(e) => match e {
                                SetupErrors::CouldNotCreateChannel => {
                                    info!("couldn't create channel");
                                }
                                SetupErrors::CouldNotSendMessage => {
                                    info!("couldn't send message");
                                }
                                SetupErrors::CouldNotCreateRole => {
                                    info!("couldn't create role");
                                }
                            },
                        }
                    }
                }
                InteractionType::MessageComponent => {
                    let InteractionData::MessageComponent(message_component) = data else {
                        return;
                    };

                    info!("message component");

                    let extra_length = 5; // this is for #1234
                    let min_length = Some(2 + extra_length); // according to embark's website min characters is 2
                    let max_length = Some(16 + extra_length); // max characters is 16

                    if message_component.custom_id == "verify" {
                        let embark_id_input = Component::TextInput(TextInput {
                            id: None,
                            custom_id: "embark_verification".to_string(),
                            label: "EMBARKID".to_string(),
                            max_length,
                            min_length,
                            placeholder: Some("name#1234".to_string()),
                            required: Some(true),
                            style: TextInputStyle::Short,
                            value: None,
                        });

                        let action_row = Component::ActionRow(ActionRow {
                            id: None,
                            components: vec![embark_id_input],
                        });

                        let data = InteractionResponseData {
                            custom_id: Some("embark_verification".to_string()),
                            title: Some("Provide Your EmbarkID".to_string()),
                            components: Some(vec![action_row]),
                            ..Default::default()
                        };

                        let response = InteractionResponse {
                            kind: InteractionResponseType::Modal,
                            data: Some(data),
                        };

                        context
                            .client
                            .interaction(context.application_id)
                            .create_response(interaction.id, &interaction.token, &response)
                            .await;
                    }
                }
                InteractionType::ApplicationCommandAutocomplete => {}
                InteractionType::ModalSubmit => {
                    info!("ModalSubmit");

                    let InteractionData::ModalSubmit(modal_submit) = data else {
                        return;
                    };

                    if modal_submit.custom_id == "embark_verification" {
                        for row in &modal_submit.components {
                            for component in &row.components {
                                if component.custom_id == "embark_verification" {
                                    info!("embark_verification");

                                    let Some(discord_user) = &interaction
                                        .member
                                        .as_ref()
                                        .and_then(|member| member.user.clone())
                                    else {
                                        return;
                                    };
                                    info!("valid user");

                                    let Some(embark_id) = component
                                        .value
                                        .as_ref()
                                        .and_then(|value| EmbarkID::new(&value).ok())
                                    else {
                                        let response = InteractionResponse {
                                            kind: InteractionResponseType::ChannelMessageWithSource,
                                            data: Some(InteractionResponseData {
                                                content: Some("Invalid EmbarkID".to_string()),
                                                flags: Some(MessageFlags::EPHEMERAL),
                                                ..Default::default()
                                            }),
                                        };

                                        context
                                            .client
                                            .interaction(context.application_id)
                                            .create_response(
                                                interaction.id,
                                                &interaction.token,
                                                &response,
                                            )
                                            .await;
                                        return;
                                    };

                                    if let Ok(embark_user_profile) =
                                        context.database.get_user_by_embark_id(&embark_id)
                                    {
                                        let response = InteractionResponse {
                                            kind: InteractionResponseType::ChannelMessageWithSource,
                                            data: Some(InteractionResponseData {
                                                content: Some(
                                                    "Someone has already claimed this EmbarkID"
                                                        .to_string(),
                                                ),
                                                flags: Some(MessageFlags::EPHEMERAL),
                                                ..Default::default()
                                            }),
                                        };

                                        context
                                            .client
                                            .interaction(context.application_id)
                                            .create_response(
                                                interaction.id,
                                                &interaction.token,
                                                &response,
                                            )
                                            .await;

                                        return;
                                    }

                                    let response = InteractionResponse {
                                        kind: InteractionResponseType::ChannelMessageWithSource,
                                        data: Some(InteractionResponseData {
                                            content: Some(format!(
                                                "You entered: {}",
                                                embark_id.to_string()
                                            )),
                                            flags: Some(MessageFlags::EPHEMERAL),
                                            ..Default::default()
                                        }),
                                    };

                                    context
                                        .client
                                        .interaction(context.application_id)
                                        .create_response(
                                            interaction.id,
                                            &interaction.token,
                                            &response,
                                        )
                                        .await;

                                    let user = User {
                                        discord_user: discord_user.id,
                                        embark_id: embark_id.clone(),
                                    };

                                    context.database.add_user(&user);

                                    if let Some(guild_id) = interaction.guild_id {
                                        update_user(
                                            &context.client,
                                            &context.cache,
                                            &user,
                                            context.database.get_guild_settings(guild_id).expect("If a user was able to click on a verifiction button it should be in the database!"),
                                        );
                                    }
                                }
                            }
                        }
                    };
                }
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

async fn setup_verification(
    context: Arc<Context>,
    guild_id: Id<GuildMarker>,
) -> Result<GuildSettings, SetupErrors> {
    let everyone_role_id = Id::new(guild_id.get());

    let role = context
        .client
        .create_role(guild_id)
        .name("verified")
        .color(6291322) // Just a green color I got from the color picker #00c822 TODO: change this maybe?
        .mentionable(false)
        .permissions(Permissions::empty())
        .await
        .map_err(|_| SetupErrors::CouldNotCreateRole)?
        .model()
        .await
        .map_err(|_| SetupErrors::CouldNotCreateRole)?;

    info!("created role: {} ({})", role.name, role.id);

    let channel = context
        .client
        .create_guild_channel(guild_id, "verify")
        .kind(ChannelType::GuildText)
        .permission_overwrites(&[
            PermissionOverwrite {
                allow: Permissions::VIEW_CHANNEL,
                deny: Permissions::SEND_MESSAGES | Permissions::ADD_REACTIONS,
                id: everyone_role_id,
                kind: PermissionOverwriteType::Role,
            },
            PermissionOverwrite {
                allow: Permissions::empty(),
                deny: Permissions::VIEW_CHANNEL,
                id: Id::new(role.id.get()),
                kind: PermissionOverwriteType::Role,
            },
        ])
        .await
        .map_err(|_| SetupErrors::CouldNotCreateChannel)?
        .model()
        .await
        .map_err(|_| SetupErrors::CouldNotCreateChannel)?;

    info!(
        "Created channel: {} ({})",
        channel.name.unwrap_or("Unknown".to_string()),
        channel.id
    );

    let message = context
        .client
        .create_message(channel.id)
        .content("verification message")
        .components(&[Component::ActionRow(
            ActionRowBuilder::new()
                .component(
                    ButtonBuilder::new(
                        twilight_model::channel::message::component::ButtonStyle::Primary,
                    )
                    .label("verify")
                    .custom_id("verify")
                    .build(),
                )
                .build(),
        )])
        .await
        .map_err(|_| SetupErrors::CouldNotSendMessage)?
        .model()
        .await
        .map_err(|_| SetupErrors::CouldNotSendMessage)?;

    info!(
        "sent message: {} in channel {}",
        message.content, channel.id
    );

    Ok(GuildSettings {
        guild_id: guild_id,
        verification_channel: channel.id,
        verified_role: role.id,
        verification_message: message.id,
    })
}

pub async fn register_setup_command(context: Arc<Context>) {
    context
        .client
        .interaction(context.application_id)
        .create_global_command()
        .chat_input("setup", "Automatically sets up the bot")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .dm_permission(false)
        .await
        .expect("error registering setup command");

    info!("Registered /setup command globally");
}

pub async fn update_user(
    client: &Client,
    cache: &InMemoryCache,
    user: &User,
    guild_config: GuildSettings,
) -> Result<(), ()> {
    let current_member = client
        .guild_member(guild_config.guild_id, user.discord_user)
        .await
        .map_err(|e| ())?
        .model()
        .await
        .map_err(|e| ())?;

    let mut new_roles = current_member.roles;
    new_roles.push(guild_config.verified_role);

    client
        .update_guild_member(guild_config.guild_id, user.discord_user)
        .nick(Some(&user.embark_id.to_string()))
        .roles(&new_roles)
        .await;

    Ok(())
}
