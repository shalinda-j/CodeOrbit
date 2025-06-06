//! Documentation agents for the CodeOrbit extension.
//!
//! This module contains agents responsible for generating and managing docs.

use crate::core::agent_registry::AgentRegistry;
use crate::core::Result;

mod docs_agent;

pub use docs_agent::DocsAgent;

/// Registers documentation agents with the provided registry.
pub fn register(registry: &mut AgentRegistry) -> Result<()> {
    registry.register(DocsAgent::new())?;
    Ok(())
}

/// Initializes all documentation agents.
pub async fn initialize() -> Result<()> {
    log::info!("Documentation agents initialized");
    Ok(())
}

/// Shuts down all documentation agents.
pub async fn shutdown() -> Result<()> {
    Ok(())
}
