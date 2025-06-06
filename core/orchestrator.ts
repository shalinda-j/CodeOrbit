import { agentRegistry } from './agentRegistry';
import { contextMemory } from './contextMemory';
import { Agent } from '../agents/base';
import { AgentNotFoundError } from './errors';

// Limit prompts to avoid flooding
const RATE_LIMIT_MS = 1000;
let lastPrompt = 0;

/** Representation of a single task assigned to an agent */
interface Subtask {
  agentName: string;
  task: string;
}

/** Central orchestrator that routes tasks between agents */
export class Orchestrator {
  /** Handle a prompt from the user and dispatch to appropriate agents */
  async receivePrompt(prompt: string): Promise<string> {
    const now = Date.now();
    if (now - lastPrompt < RATE_LIMIT_MS) {
      return 'Rate limit exceeded';
    }
    lastPrompt = now;
    const cleanPrompt = prompt.replace(/[^\w\s.,!?]/g, '');
    console.log(`[Orchestrator] received prompt: ${cleanPrompt}`);
    contextMemory.recordPrompt(cleanPrompt);

    const tasks = this.breakIntoSubtasks(prompt);
    if (tasks.length === 0) {
      console.warn('[Orchestrator] No agents matched prompt');
      return '';
    }
    const results: string[] = [];

    for (const { agentName, task } of tasks) {
      const agent = agentRegistry.getAgent(agentName);
      if (!agent) {
        console.error(new AgentNotFoundError(agentName));
        continue;
      }
      const result = await this.runAgentTask(agent, task, new Set([agentName]));
      results.push(`${agentName}: ${result}`);
      contextMemory.saveContext(agentName, { lastTask: task, lastResult: result });
    }

    return results.join('\n');
  }

  private breakIntoSubtasks(prompt: string): Subtask[] {
    const lower = prompt.toLowerCase();
    const tasks: Subtask[] = [];

    if (/(ui|frontend|react|component)/.test(lower)) {
      tasks.push({ agentName: 'frontend', task: prompt });
    }
    if (/(api|server|backend)/.test(lower)) {
      tasks.push({ agentName: 'backend', task: prompt });
    }
    if (/(db|database|schema)/.test(lower)) {
      tasks.push({ agentName: 'database', task: prompt });
    }
    if (/(deploy|ci|docker|infrastructure)/.test(lower)) {
      tasks.push({ agentName: 'devops', task: prompt });
    }
    if (/(doc|readme)/.test(lower)) {
      tasks.push({ agentName: 'docs', task: prompt });
    }

    // Default to frontend if no keywords matched
    if (tasks.length === 0) {
      tasks.push({ agentName: 'frontend', task: prompt });
    }

    return tasks;
  }

  private async runAgentTask(agent: Agent, task: string, visited: Set<string>): Promise<string> {
    if (visited.has(agent.name)) {
      console.warn(`Circular agent call detected for ${agent.name}`);
      return '';
    }
    visited.add(agent.name);
    try {
      return await agent.run(task);
    } catch (err: unknown) {
      console.error(`Agent ${agent.name} failed:`, err);
      return `error from ${agent.name}`;
    }
  }
}

export const orchestrator = new Orchestrator();

export async function sendPrompt(prompt: string) {
  return orchestrator.receivePrompt(prompt);
}
