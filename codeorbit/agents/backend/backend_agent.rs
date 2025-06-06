use crate::core::agent_registry::Agent;
use crate::core::error::Result;
use async_trait::async_trait;

/// A dummy backend agent handling server side tasks
pub struct BackendAgent {
    id: String,
}

impl BackendAgent {
    /// Create a new `BackendAgent` instance
    pub fn new() -> Self {
        Self {
            id: "backend".to_string(),
        }
    }
}

#[async_trait]
impl Agent for BackendAgent {
    fn id(&self) -> &str {
        &self.id
    }

    async fn initialize(&mut self) -> Result<()> {
        log::info!("Initializing BackendAgent");
        Ok(())
    }

    async fn process(&self, request: &str) -> Result<String> {
        log::debug!("BackendAgent processing: {}", request);
        Ok(format!("Backend response for: {}", request))
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}
