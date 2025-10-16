use crate::context::Context;
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use twilight_model::application::command::Command as ApplicationCommand;
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::application_command::{
    CommandData, CommandOptionValue,
};
use twilight_model::channel::message::{AllowedMentions, Component, Embed, MessageFlags};
use twilight_model::http::attachment::Attachment;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::marker::AttachmentMarker;
use twilight_model::id::{Id, marker::GuildMarker};

pub mod registry;

/// Supported locales for Discord commands
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Locale {
    EnglishUS,
    EnglishGB,
    Spanish,
    French,
    German,
    Japanese,
    Korean,
    Chinese,
}

impl Locale {
    pub fn as_str(&self) -> &'static str {
        match self {
            Locale::EnglishUS => "en-US",
            Locale::EnglishGB => "en-GB",
            Locale::Spanish => "es-ES",
            Locale::French => "fr",
            Locale::German => "de",
            Locale::Japanese => "ja",
            Locale::Korean => "ko",
            Locale::Chinese => "zh-CN",
        }
    }
}

impl From<Locale> for String {
    fn from(locale: Locale) -> Self {
        locale.as_str().to_string()
    }
}

#[derive(Debug, Clone)]
pub struct LocalizedText {
    pub default: String,
    pub localizations: HashMap<Locale, String>,
}

impl LocalizedText {
    pub fn new(default: impl Into<String>) -> Self {
        Self {
            default: default.into(),
            localizations: HashMap::new(),
        }
    }

    pub fn with_localization(mut self, locale: Locale, text: impl Into<String>) -> Self {
        self.localizations.insert(locale, text.into());
        self
    }

    pub fn to_discord_localizations(&self) -> Option<HashMap<String, String>> {
        if self.localizations.is_empty() {
            None
        } else {
            Some(
                self.localizations
                    .iter()
                    .map(|(locale, text)| (locale.as_str().to_string(), text.clone()))
                    .collect(),
            )
        }
    }
}

#[derive(Debug, Clone)]
pub enum CommandScope {
    Global,
    Guild(Id<GuildMarker>),
    Guilds(Vec<Id<GuildMarker>>),
}

pub struct CommandRegistration {
    pub scope: CommandScope,
    pub command: Box<dyn CommandBundle>,
}

#[derive(Clone)]
pub struct CommandContext {
    pub context: Arc<Context>,
    interaction: Interaction,
    has_replied: bool,
}

impl CommandContext {
    pub fn new(context: Arc<Context>, interaction: Interaction) -> Self {
        Self {
            context,
            interaction,
            has_replied: false,
        }
    }

    pub fn get_guild_id(&self) -> Option<Id<GuildMarker>> {
        self.interaction.guild_id
    }

    /// Get the user's preferred locale
    pub fn get_locale(&self) -> Option<Locale> {
        self.interaction
            .locale
            .as_ref()
            .and_then(|l| match l.as_str() {
                "en-US" => Some(Locale::EnglishUS),
                "en-GB" => Some(Locale::EnglishGB),
                "es-ES" => Some(Locale::Spanish),
                "fr" => Some(Locale::French),
                "de" => Some(Locale::German),
                "ja" => Some(Locale::Japanese),
                "ko" => Some(Locale::Korean),
                "zh-CN" => Some(Locale::Chinese),
                _ => None,
            })
    }

    /// Send a response to the interaction
    /// Will update the response if a response has already been given (by this function)
    pub async fn respond(&mut self, response: InteractionResponse) -> Result<(), CommandError> {
        match self.has_replied {
            true => {
                if let Some(data) = response.data {
                    // Most of these only update the new information that you give it other than
                    // attachment ids to keep
                    // not sure what to do there
                    self.update(
                        data.content.as_deref(),
                        data.embeds.as_deref(),
                        data.allowed_mentions.as_ref(),
                        data.components.as_deref(),
                        &vec![], // If you are not calling update you are **setting** the message
                        data.attachments.unwrap_or(vec![]).as_ref(),
                    )
                    .await
                } else {
                    Ok(())
                }
            }
            false => {
                self.context
                    .client
                    .interaction(self.interaction.application_id)
                    .create_response(self.interaction.id, &self.interaction.token, &response)
                    .await
                    .map_err(CommandError::Http)?;

                self.has_replied = true;

                Ok(())
            }
        }
    }

    /// Gives that says the bot is thinking
    pub async fn deferred_update_message(&mut self) -> Result<(), CommandError> {
        self.respond(InteractionResponse {
            kind: InteractionResponseType::DeferredUpdateMessage,
            data: None,
        })
        .await?;

        self.has_replied = true;

        Ok(())
    }

    /// Updates a response to the interaction
    pub async fn update(
        &self,
        content: Option<&str>,
        embeds: Option<&[Embed]>,
        allowed_mentions: Option<&AllowedMentions>,
        components: Option<&[Component]>,
        attachment_ids_to_keep: &[Id<AttachmentMarker>],
        attachments: &[Attachment],
    ) -> Result<(), CommandError> {
        self.context
            .client
            .interaction(self.interaction.application_id)
            .update_response(&self.interaction.token)
            .content(content)
            .embeds(embeds)
            .allowed_mentions(allowed_mentions)
            .components(components)
            .keep_attachment_ids(attachment_ids_to_keep)
            .attachments(attachments)
            .await
            .map_err(CommandError::Http)?;
        Ok(())
    }

    /// Reply with a public message
    pub async fn reply<S: Into<String>>(&mut self, content: S) -> Result<(), CommandError> {
        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some(content.into()),
                flags: None,
                ..Default::default()
            }),
        };
        self.respond(response).await
    }

    /// Reply with an ephemeral message
    pub async fn reply_ephemeral<S: Into<String>>(
        &mut self,
        content: S,
    ) -> Result<(), CommandError> {
        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some(content.into()),
                flags: Some(MessageFlags::EPHEMERAL),
                ..Default::default()
            }),
        };
        self.respond(response).await
    }

    /// Respond with autocomplete choices
    pub async fn autocomplete(
        &mut self,
        choices: Vec<AutocompleteChoice>,
    ) -> Result<(), CommandError> {
        let response = InteractionResponse {
            kind: InteractionResponseType::ApplicationCommandAutocompleteResult,
            data: Some(InteractionResponseData {
                choices: Some(choices.into_iter().map(Into::into).collect()),
                ..Default::default()
            }),
        };
        self.respond(response).await
    }

    /// Get a string option value
    pub fn get_string_option(&self, name: &str, data: &CommandData) -> Option<String> {
        data.options
            .iter()
            .find(|opt| opt.name == name)
            .and_then(|opt| match &opt.value {
                CommandOptionValue::String(s) => Some(s.clone()),
                _ => None,
            })
    }

    /// Get an integer option value
    pub fn get_integer_option(&self, name: &str, data: &CommandData) -> Option<i64> {
        data.options
            .iter()
            .find(|opt| opt.name == name)
            .and_then(|opt| match &opt.value {
                CommandOptionValue::Integer(i) => Some(*i),
                _ => None,
            })
    }
}

/// Autocomplete choice for command options
#[derive(Debug, Clone)]
pub struct AutocompleteChoice {
    pub name: LocalizedText,
    pub value: String,
}

impl From<AutocompleteChoice> for twilight_model::application::command::CommandOptionChoice {
    fn from(choice: AutocompleteChoice) -> Self {
        let localized_text = choice.name;

        Self {
            name: localized_text.default.clone(),
            name_localizations: localized_text.to_discord_localizations(),
            value: twilight_model::application::command::CommandOptionChoiceValue::String(
                choice.value,
            ),
        }
    }
}

/// Error type for command operations
#[derive(Debug)]
pub enum CommandError {
    Http(twilight_http::Error),
    Validation(String),
    Internal(String),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::Http(e) => write!(f, "HTTP error: {}", e),
            CommandError::Validation(msg) => write!(f, "Validation error: {}", msg),
            CommandError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for CommandError {}

/// Trait for implementing Discord slash commands
#[async_trait]
pub trait CommandBundle: Send + Sync {
    /// Get the command definition for Discord
    fn definition(&self) -> ApplicationCommand;

    /// Execute the command
    async fn execute(
        &self,
        context: &mut CommandContext,
        data: &CommandData,
    ) -> Result<(), CommandError>;

    /// Handle autocomplete for this command (optional)
    async fn autocomplete(
        &self,
        _context: CommandContext,
        _data: &CommandData,
    ) -> Result<Vec<AutocompleteChoice>, CommandError> {
        Ok(Vec::new())
    }

    /// Get the command name
    fn name(&self) -> String {
        self.definition().name.clone()
    }
}
