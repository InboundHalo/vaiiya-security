use super::{CommandBundle, CommandContext, CommandError, CommandRegistration, CommandScope};
use crate::context::Context;
use crate::handler::Handler;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};
use twilight_model::application::interaction::application_command::CommandData;
use twilight_model::application::interaction::{Interaction, InteractionData, InteractionType};
use twilight_model::gateway::event::Event;
use twilight_model::id::{Id, marker::GuildMarker};

/// Registry for managing Discord commands
pub struct CommandRegistry {
    global_commands: HashMap<Box<str>, Box<dyn CommandBundle>>,
    guild_commands: HashMap<Id<GuildMarker>, HashMap<Arc<str>, Arc<dyn CommandBundle>>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            global_commands: HashMap::new(),
            guild_commands: HashMap::new(),
        }
    }

    /// Register commands from registrations
    pub fn register_commands(&mut self, registrations: Vec<CommandRegistration>) {
        for registration in registrations {
            let command = registration.command;

            match registration.scope {
                CommandScope::Global => {
                    let name = command.name();
                    info!("Registering global command: {}", name);
                    self.global_commands.insert(name.into(), command);
                }
                CommandScope::Guild(guild_id) => {
                    let name = command.name();
                    info!("Registering guild command for {}: {}", guild_id, name);
                    self.guild_commands
                        .entry(guild_id)
                        .or_default()
                        .insert(name.into(), Arc::from(command));
                }
                CommandScope::Guilds(guild_ids) => {
                    let name: Arc<str> = command.name().into();
                    let command: Arc<dyn CommandBundle> = command.into();
                    for guild_id in guild_ids {
                        info!("Registering guild command for {}: {}", guild_id, name);
                        self.guild_commands
                            .entry(guild_id)
                            .or_default()
                            .insert(Arc::clone(&name), Arc::clone(&command));
                    }
                }
            }
        }
    }

    /// Deploy all commands to Discord
    pub async fn deploy(&self, context: &Context) -> Result<(), Box<dyn std::error::Error>> {
        // Deploy global commands
        if !self.global_commands.is_empty() {
            let definitions: Vec<_> = self
                .global_commands
                .values()
                .map(|cmd| cmd.definition())
                .collect();

            info!("Deploying {} global commands", definitions.len());
            context
                .client
                .interaction(context.application_id)
                .set_global_commands(&definitions)
                .await?;
        }

        // Deploy guild commands
        for (guild_id, commands) in &self.guild_commands {
            let definitions: Vec<_> = commands.values().map(|cmd| cmd.definition()).collect();

            info!(
                "Deploying {} commands to guild {}",
                definitions.len(),
                guild_id
            );
            context
                .client
                .interaction(context.application_id)
                .set_guild_commands(*guild_id, &definitions)
                .await?;
        }

        Ok(())
    }

    /// Find a command by name
    fn find_command(
        &self,
        name: &str,
        guild_id: Option<Id<GuildMarker>>,
    ) -> Option<&dyn CommandBundle> {
        // First check guild commands if guild_id is provided
        if let Some(guild_id) = guild_id {
            if let Some(guild_commands) = self.guild_commands.get(&guild_id) {
                if let Some(command) = guild_commands.get(name) {
                    return Some(&**command);
                }
            }
        }

        // Fall back to global commands
        self.global_commands
            .get(name)
            .and_then(|command| Some(command.as_ref()))
    }

    pub async fn handle_interaction(&self, ctx: Arc<Context>, interaction: &Interaction) {
        let command_ctx = CommandContext::new(Arc::clone(&ctx), interaction.clone());

        match interaction.kind {
            InteractionType::ApplicationCommand => {
                if let Some(InteractionData::ApplicationCommand(data)) = &interaction.data {
                    self.handle_command(command_ctx, data).await;
                }
            }
            InteractionType::ApplicationCommandAutocomplete => {
                if let Some(InteractionData::ApplicationCommand(data)) = &interaction.data {
                    self.handle_autocomplete(command_ctx, data).await;
                }
            }
            _ => {}
        }
    }

    async fn handle_command(
        &self,
        mut command_context: CommandContext,
        data: &twilight_model::application::interaction::application_command::CommandData,
    ) {
        if let Some(command) = self.find_command(&data.name, data.guild_id) {
            if let Err(error) = command.execute(&mut command_context, data).await {
                error!("Command execution error: {}", error);

                match error {
                    CommandError::Http(message) => {
                        let message = format!("HTTP error: {}", message);

                        if command_context.reply_ephemeral(&message).await.is_err() {
                            error!("Command reply error: {}", message);
                        }
                    }
                    CommandError::Validation(message) => {
                        let message = format!("Command validation error: {}", message);

                        if command_context.reply_ephemeral(&message).await.is_err() {
                            error!("Command validation error: {}", message);
                        }
                    }
                    CommandError::Internal(message) => {
                        error!("Command internal error: {}", message);

                        if command_context.reply_ephemeral(&message).await.is_err() {
                            error!("Command internal error: {}", message);
                        }
                    }
                }
            }
        }
    }

    async fn handle_autocomplete(&self, ctx: CommandContext, data: &CommandData) {
        if let Some(command) = self.find_command(&data.name, data.guild_id) {
            match command.autocomplete(ctx, data).await {
                Ok(_choices) => {
                    // Response is handled by the command's autocomplete method
                }
                Err(error) => {
                    error!("Autocomplete error: {}", error);
                }
            }
        }
    }
}

#[async_trait]
impl Handler for CommandRegistry {
    async fn handle(&self, ctx: Arc<Context>, event: Arc<Event>) {
        if let Event::InteractionCreate(interaction) = &*event {
            self.handle_interaction(ctx, &interaction.0).await;
        }
    }
}
