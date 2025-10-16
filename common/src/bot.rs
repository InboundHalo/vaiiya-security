use crate::commands::registry::CommandRegistry;
use crate::context::Context;
use crate::handler::Handler;
use std::sync::Arc;
use tracing::{debug, info};
use twilight_cache_inmemory::{CacheableStageInstance, DefaultInMemoryCache};
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt};
use twilight_http::Client;

pub struct Bot {
    handlers: Vec<Arc<Box<dyn Handler + Send>>>,
    token: String,
}

impl Bot {
    pub fn new(token: String) -> Self {
        Self {
            handlers: Vec::new(),
            token,
        }
    }

    pub fn register<H: Handler + Send + 'static>(&mut self, handler: H) {
        self.handlers.push(Arc::new(Box::new(handler)));
    }

    pub fn start(mut self) {
        info!("Starting the bot");

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        debug!("Made runtime");

        rt.block_on(async {
            debug!("Creating client, cache & context");
            let client = Client::new(self.token.clone());
            let cache = DefaultInMemoryCache::new();

            let current_user_application = client
                .current_user_application()
                .await
                .expect("Could not get the application_id!")
                .model()
                .await
                .expect("Could not get the application_id!");

            let application_id = current_user_application.id;

            let bot = current_user_application
                .bot
                .expect("Not bot associated with the discord app ID");

            let context = Arc::new(Context {
                client,
                cache,
                application_id,
                bot,
            });

            info!("Connecting...");

            let mut shard = Shard::new(ShardId::ONE, self.token.to_string(), Intents::all());

            self.ready_commands(&context)
                .await
                .expect("Couldn't ready commands");

            while let Some(Ok(event)) = shard.next_event(EventTypeFlags::all()).await {
                let event = Arc::new(event);

                self.dispatch(Arc::clone(&context), Arc::clone(&event));

                context.cache.update(&*event);
            }
        });
    }

    pub async fn start_blocking(self) {
        tokio::task::spawn_blocking(move || {
            self.start();
        });
    }

    async fn ready_commands(
        &mut self,
        context: &Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut command_registry = CommandRegistry::new();
        let mut all_registrations = Vec::new();

        info!("Registering commands from handlers...");

        // Register commands from handlers (new system)
        for handler in &self.handlers {
            let registrations = handler.commands();

            for command_registration in &registrations {
                info!(
                    "Registering command from handler: {}",
                    command_registration.command.name()
                );
            }

            all_registrations.extend(registrations);
        }

        command_registry.register_commands(all_registrations);
        command_registry.deploy(&context).await?;

        self.register(command_registry);

        Ok(())
    }

    fn dispatch(&self, context: Arc<Context>, event: Arc<Event>) {
        for handler in &self.handlers {
            let cloned_handler = Arc::clone(handler);
            let cloned_context = Arc::clone(&context);
            let cloned_event = Arc::clone(&event);

            tokio::spawn(async move {
                cloned_handler.handle(cloned_context, cloned_event).await;
            });
        }
    }
}
