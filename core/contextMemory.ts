/** Simple in-memory store for agent context */
export class ContextMemory {
  /** In-memory store keyed by agent name */
  private readonly store = new Map<string, unknown>();
  /** History of recent prompts for adaptive memory */
  private readonly history: {prompt: string; timestamp: number}[] = [];

  /**
   * Persist context for an agent. Overwrites any existing entry.
   */
  saveContext(agentName: string, data: unknown): void {
    this.store.set(agentName, data);
  }

  recordPrompt(prompt: string): void {
    this.history.push({ prompt, timestamp: Date.now() });
    if (this.history.length > 5) {
      this.history.shift();
    }
  }

  /**
   * Retrieve previously saved context for an agent.
   */
  getContext<T>(agentName: string): T | undefined {
    return this.store.get(agentName) as T | undefined;
  }

  getHistory() {
    return [...this.history];
  }
}

export const contextMemory = new ContextMemory();
