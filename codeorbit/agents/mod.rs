//! Agents module for the CodeOrbit extension.
//!
//! This module contains all the agents that make up the CodeOrbit system.

pub mod backend;
pub mod database;
pub mod devops;
pub mod docs;
pub mod frontend;

use crate::core::agent_registry::AgentRegistry;
use crate::core::Result;

/// Initializes all agents.
pub async fn initialize() -> Result<()> {
    // Initialize all agents here
    frontend::initialize().await?;
    backend::initialize().await?;
    database::initialize().await?;
    devops::initialize().await?;
    docs::initialize().await?;

    Ok(())
}

/// Registers all agents into the provided registry.
pub fn register_all(registry: &mut AgentRegistry) -> Result<()> {
    frontend::register(registry)?;
    backend::register(registry)?;
    database::register(registry)?;
    devops::register(registry)?;
    docs::register(registry)?;
    Ok(())
}
