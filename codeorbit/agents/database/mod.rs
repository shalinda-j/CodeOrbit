//! Database agents for the CodeOrbit extension.
//!
//! This module contains agents that handle database-related tasks such as
//! schema management and migrations.

use crate::core::agent_registry::AgentRegistry;
use crate::core::Result;

mod database_agent;

pub use database_agent::DatabaseAgent;

/// Registers database agents with the provided registry.
pub fn register(registry: &mut AgentRegistry) -> Result<()> {
    registry.register(DatabaseAgent::new())?;
    Ok(())
}

/// Initializes all database agents.
pub async fn initialize() -> Result<()> {
    log::info!("Database agents initialized");
    Ok(())
}

/// Shuts down all database agents.
pub async fn shutdown() -> Result<()> {
    Ok(())
}
