use async_trait::async_trait;
use common::commands::CommandBundle;
use common::commands::CommandContext;
use common::commands::CommandError;
use common::commands::CommandRegistration;
use data::EmbarkID;
use data::GuildSettings;
use data::User;
use twilight_http::Client;
use twilight_model::application::command::Command;
use twilight_model::application::command::CommandType;
use twilight_model::application::interaction::application_command::CommandData;
use twilight_model::application::interaction::{InteractionData, InteractionType};
use twilight_model::channel::ChannelType;
use twilight_model::channel::message::component::{ActionRow, TextInput, TextInputStyle};
use twilight_model::channel::message::{Component, MessageFlags};
use twilight_model::channel::permission_overwrite::{PermissionOverwrite, PermissionOverwriteType};
use twilight_model::guild::Permissions;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::marker::InteractionMarker;
use twilight_util::builder::command::CommandBuilder;
use twilight_util::builder::message::{ActionRowBuilder, ButtonBuilder};

use common::context;
use common::handler::Handler;
use data::Database;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use twilight_gateway::Event;
use twilight_util::permission_calculator::PermissionCalculator;

use crate::context::Context;
mod guild_welcome;

pub struct EmbarkIDSync {
    database: Arc<Database>,
}

impl EmbarkIDSync {
    pub async fn new(database: Arc<Database>) -> Self {
        EmbarkIDSync { database }
    }
}

#[async_trait]
impl Handler for EmbarkIDSync {
    async fn handle(&self, context: Arc<Context>, event: Arc<Event>) {
        match &*event {
            Event::MemberAdd(member_add) => {
                let user = &member_add.user;
                let guild_id = member_add.guild_id;

                let Some(guild_config) = self.database.get_guild_settings(&guild_id) else {
                    return;
                };

                let guild_name = context
                    .cache()
                    .guild(guild_id)
                    .and_then(|cached_guild| Some(cached_guild.name().to_string()))
                    .unwrap_or("a guild".to_string());

                match self.database.get_user_by_discord_id(user.id) {
                    None => {
                        // TODO: DM the user
                        context.send_dm_to_user(user.id,format!("`{}` uses this bot for Embark ID linking.\nPlease go to <#{}> and follow the instructions to link your account.", guild_name, guild_config.verification_channel).as_str()).await;
                    }
                    Some(database_user) => {
                        update_user(&context.client(), &database_user, &guild_config).await;

                        if let Some(other_guilds) = context.cache().user_guilds(user.id) {
                            let guild_settings_list: Vec<GuildSettings> = other_guilds
                                .value()
                                .iter()
                                .filter_map(|guild_id| self.database.get_guild_settings(guild_id))
                                .collect();

                            for guild_settings in guild_settings_list {
                                update_user(&context.client(), &database_user, &guild_settings)
                                    .await;
                            }
                        }

                        context.send_dm_to_user(user.id,format!(
                                             "`{}` uses this bot for Embark ID linking.
                                             \nSince your have already linked your account (`{}`) there is nothing that you need to do. GLHF Contestant!",
                                             guild_name,
                                             database_user.embark_id.to_string()).as_str())
                                     .await;
                    }
                }
            }
            Event::GuildCreate(guild_create) => {
                self.guild_event(context, guild_create).await;
            }
            Event::InteractionCreate(interaction) => {
                let Some(data) = &interaction.data else {
                    return;
                };

                match interaction.kind {
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
                    InteractionType::ApplicationCommand => {}
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
                                            reply_ephemeral(
                                                &context,
                                                interaction.id,
                                                &interaction.token,
                                                "Invalid EmbarkID".to_string(),
                                            )
                                            .await;

                                            return;
                                        };

                                        if let Some(embark_user_profile) =
                                            self.database.get_user_by_embark_id(&embark_id)
                                        {
                                            reply_ephemeral(
                                                &context,
                                                interaction.id,
                                                &interaction.token,
                                                "Someone has already claimed this EmbarkID".into(),
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

                                        self.database.add_user(&user);

                                        if let Some(guild_id) = interaction.guild_id {
                                            let Some(guild_settings) =
                                                self.database.get_guild_settings(&guild_id)
                                            else {
                                                return;
                                            };

                                            update_user(&context.client, &user, &guild_settings)
                                                .await;
                                        }
                                    }
                                }
                            }
                        };
                    }
                    _ => warn!("unknown interaction type"),
                }
            }
            _ => {}
        }
    }

    fn commands(&self) -> Vec<CommandRegistration> {
        vec![CommandRegistration {
            scope: common::commands::CommandScope::Global,
            command: Box::new(SetupCommand::new(Arc::clone(&self.database))),
        }]
    }
}

struct SetupCommand {
    database: Arc<Database>,
}

impl SetupCommand {
    fn new(database: Arc<Database>) -> Self {
        SetupCommand { database }
    }

    pub async fn setup_verification(
        context: &Arc<Context>,
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
}

#[async_trait]
impl CommandBundle for SetupCommand {
    fn definition(&self) -> Command {
        CommandBuilder::new(
            "setup",
            "Automatically sets up the bot",
            CommandType::ChatInput,
        )
        .build()
    }

    /// Execute the command
    async fn execute(
        &self,
        context: &mut CommandContext,
        data: &CommandData,
    ) -> Result<(), CommandError> {
        let Some(guild_id) = context.get_guild_id() else {
            context
                .reply("This command must be done in a guild!")
                .await?;
            return Ok(());
        };

        match SetupCommand::setup_verification(&context.context, guild_id).await {
            Ok(guild_settings) => {
                self.database
                    .set_guild_settings(&guild_settings)
                    .map_err(|_| CommandError::Internal("Could not save guild settings!".into()))?;
                context.reply("Setup complete!").await?;
            }
            Err(setup_errors) => match setup_errors {
                SetupErrors::CouldNotCreateChannel => {
                    context.reply("Could not create channel").await?;
                }

                SetupErrors::CouldNotSendMessage => {
                    context.reply("Could not send message!").await?
                }
                SetupErrors::CouldNotCreateRole => context.reply("Could not create role").await?,
            },
        }

        Ok(())
    }
}

enum SetupErrors {
    CouldNotCreateChannel,
    CouldNotSendMessage,
    CouldNotCreateRole,
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
    user: &User,
    guild_config: &GuildSettings,
) -> Result<(), ()> {
    let current_member = client
        .guild_member(guild_config.guild_id, user.discord_user)
        .await
        .map_err(|_| ())?
        .model()
        .await
        .map_err(|_| ())?;

    let mut new_roles = current_member.roles;
    new_roles.push(guild_config.verified_role);

    client
        .update_guild_member(guild_config.guild_id, user.discord_user)
        .nick(Some(&user.embark_id.to_string()))
        .roles(&new_roles)
        .await
        .map_err(|_| ())?;

    Ok(())
}

pub async fn reply_ephemeral(
    context: &Arc<Context>,
    interaction_id: Id<InteractionMarker>,
    interaction_token: &str,
    content: String,
) {
    let response = InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(InteractionResponseData {
            content: Some(content.clone().into()),
            flags: Some(MessageFlags::EPHEMERAL),
            ..Default::default()
        }),
    };
    let response = context
        .client
        .interaction(context.application_id)
        .create_response(interaction_id, interaction_token, &response)
        .await;

    match response {
        Ok(_) => debug!("Created response: {}", content),
        Err(error) => error!("Could not create response: {} | Error: {}", content, error),
    };
}
