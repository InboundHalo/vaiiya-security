use crate::commands::CommandRegistration;
use crate::context::Context;
use async_trait::async_trait;
use std::sync::Arc;
use twilight_gateway::Event;

#[async_trait]
pub trait Handler: Send + Sync {
    async fn handle(&self, context: Arc<Context>, event: Arc<Event>);

    /// Return commands this handler wants to register
    fn commands(&self) -> Vec<CommandRegistration> {
        Vec::new()
    }
}
