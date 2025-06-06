/** Simple in-memory store for agent context */
export class ContextMemory {
  /** In-memory store keyed by agent name */
  private readonly store = new Map<string, unknown>();

  /**
   * Persist context for an agent. Overwrites any existing entry.
   */
  saveContext(agentName: string, data: unknown): void {
    this.store.set(agentName, data);
  }

  /**
   * Retrieve previously saved context for an agent.
   */
  getContext<T>(agentName: string): T | undefined {
    return this.store.get(agentName) as T | undefined;
  }
}

export const contextMemory = new ContextMemory();
