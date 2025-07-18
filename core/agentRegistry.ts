import { Agent } from '../agents/base';

export { Agent };

/** Global registry holding all available agents */
class AgentRegistry {
  private agents = new Map<string, Agent>();

  registerAgent(agent: Agent) {
    this.agents.set(agent.name, agent);
  }

  getAgent(name: string): Agent | undefined {
    return this.agents.get(name);
  }

  listAgents(): string[] {
    return Array.from(this.agents.keys());
  }
}

export const agentRegistry = new AgentRegistry();
