# Agent Architecture

CodeOrbit uses a modular agent design. Each agent implements a common `Agent` interface with `run` and `getCapabilities` methods. The `agentRegistry` stores active agents and the `Orchestrator` routes tasks to them. `ContextMemory` holds recent context so agents can adapt to previous prompts.
