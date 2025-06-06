//! DevOps agents for the CodeOrbit extension.
//!
//! This module contains agents that handle CI/CD and infrastructure tasks.

use crate::core::agent_registry::AgentRegistry;
use crate::core::Result;

mod devops_agent;

pub use devops_agent::DevOpsAgent;

/// Registers DevOps agents with the provided registry.
pub fn register(registry: &mut AgentRegistry) -> Result<()> {
    registry.register(DevOpsAgent::new())?;
    Ok(())
}

/// Initializes all DevOps agents.
pub async fn initialize() -> Result<()> {
    log::info!("DevOps agents initialized");
    Ok(())
}

/// Shuts down all DevOps agents.
pub async fn shutdown() -> Result<()> {
    Ok(())
}
