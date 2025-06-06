import { agentRegistry } from './agentRegistry';
import { contextMemory } from './contextMemory';
import { Agent } from '../agents/base';
import { AgentNotFoundError } from './errors';

/** Representation of a single task assigned to an agent */
interface Subtask {
  agentName: string;
  task: string;
}

/** Central orchestrator that routes tasks between agents */
export class Orchestrator {
  /** Handle a prompt from the user and dispatch to appropriate agents */
  async receivePrompt(prompt: string): Promise<string> {
    console.log(`[Orchestrator] received prompt: ${prompt}`);

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
      const result = await this.runAgentTask(agent, task);
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

  private async runAgentTask(agent: Agent, task: string): Promise<string> {
    try {
      return await agent.run(task);
    } catch (err: unknown) {
      console.error(`Agent ${agent.name} failed:`, err);
      return `error from ${agent.name}`;
    }
  }
}
