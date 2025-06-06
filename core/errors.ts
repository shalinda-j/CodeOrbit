export class AgentError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'AgentError';
  }
}

export class AgentNotFoundError extends AgentError {
  constructor(agentName: string) {
    super(`Agent "${agentName}" not registered`);
    this.name = 'AgentNotFoundError';
  }
}
