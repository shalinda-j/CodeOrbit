import { Agent } from '../agents/base';
import { AgentError } from './errors';

/** Global registry holding all available agents */
class AgentRegistry {
  private readonly agents = new Map<string, Agent>();

  registerAgent(name: string, agent: Agent): void {
    if (this.agents.has(name)) {
      throw new AgentError(`Agent "${name}" already registered`);
    }
    this.agents.set(name, agent);
  }

  getAgent(name: string): Agent | undefined {
    const agent = this.agents.get(name);
    if (!agent) {
      return this.agents.get('frontend');
    }
    return agent;
  }

  listAgents(): string[] {
    return Array.from(this.agents.keys());
  }
}

export const agentRegistry = new AgentRegistry();
