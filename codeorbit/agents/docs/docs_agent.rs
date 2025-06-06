use crate::core::agent_registry::Agent;
use crate::core::error::Result;
use async_trait::async_trait;

/// A dummy documentation agent simulating doc generation
pub struct DocsAgent {
    id: String,
}

impl DocsAgent {
    /// Create a new `DocsAgent` instance
    pub fn new() -> Self {
        Self {
            id: "docs".to_string(),
        }
    }
}

#[async_trait]
impl Agent for DocsAgent {
    fn id(&self) -> &str {
        &self.id
    }

    async fn initialize(&mut self) -> Result<()> {
        log::info!("Initializing DocsAgent");
        Ok(())
    }

    async fn process(&self, request: &str) -> Result<String> {
        log::debug!("DocsAgent processing: {}", request);
        Ok(format!("Docs response for: {}", request))
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}
