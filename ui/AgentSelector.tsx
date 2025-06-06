import React from 'react';
import { agentRegistry } from '../core/agentRegistry';

/** Allows the user to view registered agents */
export function AgentSelector() {
  const agents = agentRegistry.listAgents();
  return (
    <div className="agent-selector border p-2 mb-2 rounded">
      <strong>Agents</strong>
      <ul>
        {agents.map(name => <li key={name}>{name}</li>)}
      </ul>
    </div>
  );
}
