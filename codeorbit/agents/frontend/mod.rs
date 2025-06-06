//! Frontend agents for the CodeOrbit extension.
//!
//! This module contains agents that handle frontend-related tasks such as
//! UI generation and component creation.

use crate::core::agent_registry::AgentRegistry;
use crate::core::Result;

pub mod component_generator_agent;
pub mod ui_planner_agent;

/// Registers frontend agents with the provided registry.
pub fn register(registry: &mut AgentRegistry) -> Result<()> {
    registry.register(ui_planner_agent::UiPlannerAgent::new())?;
    registry.register(component_generator_agent::ComponentGeneratorAgent::new())?;
    Ok(())
}

/// Initializes all frontend agents.
pub async fn initialize() -> Result<()> {
    log::info!("Frontend agents initialized");
    Ok(())
}

/// Shuts down all frontend agents.
pub async fn shutdown() -> Result<()> {
    Ok(())
}
