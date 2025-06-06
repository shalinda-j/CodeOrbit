//! Backend agents for the CodeOrbit extension.
//!
//! This module contains agents that handle backend-related tasks such as
//! API integration, data processing, and server-side logic.

use crate::core::agent_registry::AgentRegistry;
use crate::core::Result;

mod backend_agent;

pub use backend_agent::BackendAgent;

/// Registers backend agents with the provided registry.
pub fn register(registry: &mut AgentRegistry) -> Result<()> {
    registry.register(BackendAgent::new())?;
    Ok(())
}

/// Initializes all backend agents.
pub async fn initialize() -> Result<()> {
    // Placeholder for future initialization logic
    log::info!("Backend agents initialized");
    Ok(())
}

/// Shuts down all backend agents.
pub async fn shutdown() -> Result<()> {
    Ok(())
}
