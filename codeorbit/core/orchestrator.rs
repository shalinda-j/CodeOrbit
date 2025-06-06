//! The orchestrator is the central component that manages agent communication and task coordination.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::agents;
use crate::core::agent_registry::AgentRegistry;
use crate::core::context::Context;
use crate::core::error::Result;

/// Representation of a subtask destined for an agent
struct Subtask {
    agent_name: String,
    task: String,
}

/// The Orchestrator manages the lifecycle of agents and coordinates their activities.
pub struct Orchestrator {
    agent_registry: AgentRegistry,
    context: Context,
    last_prompt: Option<Instant>,
}

impl Orchestrator {
    /// Creates a new instance of the Orchestrator.
    pub fn new() -> Self {
        Self {
            agent_registry: AgentRegistry::new(),
            context: Context::new(),
            last_prompt: None,
        }
    }

    /// Initializes the orchestrator and all registered agents.
    pub async fn initialize(&mut self) -> Result<()> {
        log::info!("Initializing CodeOrbit Orchestrator");
        // Register built-in agents
        agents::register_all(&mut self.agent_registry)?;

        self.agent_registry.initialize().await?;
        Ok(())
    }

    /// Handles a prompt originating from the UI and returns the aggregated agent response.
    pub async fn handle_user_prompt(&mut self, prompt: &str) -> Result<String> {
        // rate limit simple flood
        const RATE_LIMIT_MS: u128 = 1000;
        if let Some(last) = self.last_prompt {
            if last.elapsed() < Duration::from_millis(RATE_LIMIT_MS as u64) {
                return Ok("Rate limit exceeded".to_string());
            }
        }
        self.last_prompt = Some(Instant::now());

        let sanitized = prompt.replace(|c: char| !c.is_ascii() || c == '<' || c == '>', "");
        log::info!("User prompt: {}", sanitized);
        self.context.record_prompt(&sanitized);

        let tasks = self.break_into_subtasks(&sanitized);
        if tasks.is_empty() {
            log::warn!("No agents matched prompt");
            return Ok(String::new());
        }

        let mut results = Vec::new();
        for sub in tasks {
            let mut visited = HashSet::new();
            let result = self
                .run_agent_task(&sub.agent_name, &sub.task, &mut visited)
                .await?;
            if !result.is_empty() {
                results.push(format!("{}: {}", sub.agent_name, result));
                let key = format!("{}_last_result", sub.agent_name);
                let _ = self.context.set(&key, &result);
            }
        }
        Ok(results.join("\n"))
    }

    fn break_into_subtasks(&self, prompt: &str) -> Vec<Subtask> {
        let lower = prompt.to_lowercase();
        let mut tasks = Vec::new();

        if lower.contains("ui") || lower.contains("frontend") {
            tasks.push(Subtask {
                agent_name: "frontend".into(),
                task: prompt.into(),
            });
        }
        if lower.contains("backend") || lower.contains("api") {
            tasks.push(Subtask {
                agent_name: "backend".into(),
                task: prompt.into(),
            });
        }
        if lower.contains("database") || lower.contains("db") {
            tasks.push(Subtask {
                agent_name: "database".into(),
                task: prompt.into(),
            });
        }
        if lower.contains("deploy") || lower.contains("ci") {
            tasks.push(Subtask {
                agent_name: "devops".into(),
                task: prompt.into(),
            });
        }
        if lower.contains("doc") {
            tasks.push(Subtask {
                agent_name: "docs".into(),
                task: prompt.into(),
            });
        }

        if tasks.is_empty() {
            tasks.push(Subtask {
                agent_name: "frontend".into(),
                task: prompt.into(),
            });
        }

        tasks
    }

    async fn run_agent_task(
        &self,
        agent_name: &str,
        task: &str,
        visited: &mut HashSet<String>,
    ) -> Result<String> {
        if !visited.insert(agent_name.to_string()) {
            log::warn!("Circular call detected for {}", agent_name);
            return Ok(String::new());
        }

        match self.agent_registry.get_or_default(agent_name) {
            Some(agent) => match agent.process(task).await {
                Ok(r) => Ok(r),
                Err(e) => {
                    log::error!("Agent {} failed: {}", agent_name, e);
                    Ok(String::new())
                }
            },
            None => {
                log::error!("Agent not found: {}", agent_name);
                Ok(String::new())
            }
        }
    }

    /// Shuts down the orchestrator and all agents.
    pub async fn shutdown(&mut self) -> Result<()> {
        log::info!("Shutting down CodeOrbit Orchestrator");
        self.agent_registry.shutdown().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_initialization() {
        let mut orchestrator = Orchestrator::new();
        assert!(orchestrator.initialize().await.is_ok());
    }

    #[tokio::test]
    async fn test_handle_user_prompt() {
        let mut orchestrator = Orchestrator::new();
        orchestrator.initialize().await.unwrap();
        let response = orchestrator
            .handle_user_prompt("Create login page")
            .await
            .unwrap();
        assert!(!response.is_empty());
    }
}
