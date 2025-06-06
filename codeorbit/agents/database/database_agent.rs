use crate::core::agent_registry::Agent;
use crate::core::error::Result;
use async_trait::async_trait;

/// A dummy database agent handling DB tasks
pub struct DatabaseAgent {
    id: String,
}

impl DatabaseAgent {
    /// Create a new `DatabaseAgent` instance
    pub fn new() -> Self {
        Self {
            id: "database".to_string(),
        }
    }
}

#[async_trait]
impl Agent for DatabaseAgent {
    fn id(&self) -> &str {
        &self.id
    }

    async fn initialize(&mut self) -> Result<()> {
        log::info!("Initializing DatabaseAgent");
        Ok(())
    }

    async fn process(&self, request: &str) -> Result<String> {
        log::debug!("DatabaseAgent processing: {}", request);
        Ok(format!("Database response for: {}", request))
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}
