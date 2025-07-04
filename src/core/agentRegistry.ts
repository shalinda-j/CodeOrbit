import { IAgent, AgentResult, Subtask } from './Agent';

/**
 * Registry for managing all agents in the system.
 * Implements the singleton pattern to ensure a single source of truth.
 */
export class AgentRegistry {
  private static instance: AgentRegistry;
  private agents: Map<string, IAgent> = new Map();

  /**
   * Private constructor to enforce singleton pattern
   */
  private constructor() {}

  /**
   * Get the singleton instance of the AgentRegistry
   */
  public static getInstance(): AgentRegistry {
    if (!AgentRegistry.instance) {
      AgentRegistry.instance = new AgentRegistry();
    }
    return AgentRegistry.instance;
  }

  /**
   * Register a new agent with the registry
   * @param agent The agent to register
   * @throws Error if an agent with the same ID is already registered
   */
  public registerAgent(agent: IAgent): void {
    if (this.agents.has(agent.id)) {
      throw new Error(`Agent with ID '${agent.id}' is already registered`);
    }
    
    console.log(`[AgentRegistry] Registering agent: ${agent.name} (${agent.id})`);
    this.agents.set(agent.id, agent);
  }

  /**
   * Get an agent by its ID
   * @param agentId The ID of the agent to retrieve
   * @returns The agent if found, undefined otherwise
   */
  public getAgent(agentId: string): IAgent | undefined {
    return this.agents.get(agentId);
  }

  /**
   * Get all registered agents
   * @returns Array of all registered agents
   */
  public getAgents(): AgentBase[] {
    return Array.from(this.agents.values());
  }

  /**
   * Find agents that have all the specified capabilities
   * @param capabilities Array of capability strings to match
   * @returns Array of agents that have all specified capabilities
   */
  public findAgentsByCapabilities(capabilities: string[]): AgentBase[] {
    return this.getAgents().filter(agent =>
      capabilities.every(capability => 
        agent.getCapabilities().includes(capability)
      )
    );
  }

  /**
   * Execute a task with the specified agent
   * @param agentId The ID of the agent to execute the task
   * @param input The input for the task
   * @param context Optional context data
   * @returns Promise that resolves to the task result
   */
  public async executeWithAgent(
    agentId: string,
    input: string,
    context?: Record<string, any>
  ): Promise<AgentResult> {
    const agent = this.getAgent(agentId);
    if (!agent) {
      return {
        success: false,
        output: `Agent with ID '${agentId}' not found`,
        error: new Error(`Agent not found: ${agentId}`)
      };
    }

    console.log(`[AgentRegistry] Executing with agent: ${agent.name} (${agentId})`);
    console.log(`[AgentRegistry] Input: ${input.substring(0, 100)}${input.length > 100 ? '...' : ''}`);
    
    try {
      const result = await agent.run(input, context);
      console.log(`[AgentRegistry] Agent ${agent.name} execution ${result.success ? 'succeeded' : 'failed'}`);
      return result;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      console.error(`[AgentRegistry] Error executing agent ${agentId}:`, errorMessage);
      return {
        success: false,
        output: `Error executing agent: ${errorMessage}`,
        error: error instanceof Error ? error : new Error(errorMessage)
      };
    }
  }

  /**
   * Execute subtasks in parallel
   * @param subtasks Array of subtasks to execute
   * @returns Promise that resolves when all subtasks are complete
   */
  public async executeSubtasks(subtasks: Subtask[]): Promise<AgentResult[]> {
    if (subtasks.length === 0) {
      return [];
    }

    console.log(`[AgentRegistry] Executing ${subtasks.length} subtasks in parallel`);
    
    // Sort subtasks by priority (higher priority first)
    const sortedSubtasks = [...subtasks].sort((a, b) => (b.priority || 0) - (a.priority || 0));
    
    // Execute subtasks in parallel
    const results = await Promise.all(
      sortedSubtasks.map(async subtask => {
        console.log(`[AgentRegistry] Executing subtask with agent: ${subtask.agentId}`);
        const result = await this.executeWithAgent(
          subtask.agentId,
          subtask.input,
          subtask.context
        );
        
        // If the agent has a handler for subtask results, call it
        const agent = this.getAgent(subtask.agentId);
        if (agent && agent.handleSubtaskResult) {
          await agent.handleSubtaskResult(result, subtask.context || {});
        }
        
        return result;
      })
    );
    
    return results;
  }

  /**
   * Clear all registered agents (for testing)
   */
  public clear(): void {
    console.log('[AgentRegistry] Clearing all registered agents');
    this.agents.clear();
  }

  /**
   * Get the number of registered agents
   */
  public get size(): number {
    return this.agents.size;
  }
}

// Export a singleton instance
export const agentRegistry = AgentRegistry.getInstance();
