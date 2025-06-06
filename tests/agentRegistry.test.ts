import { describe, it, expect } from 'vitest';
import { agentRegistry } from '../core/agentRegistry';
import { Agent } from '../agents/base';

class DummyAgent implements Agent {
  name = 'dummy';
  async run() { return 'ok'; }
  getCapabilities() { return []; }
}

describe('agentRegistry', () => {
  it('registers and retrieves agents', () => {
    const agent = new DummyAgent();
    agentRegistry.registerAgent('dummy', agent);
    expect(agentRegistry.getAgent('dummy')).toBe(agent);
  });
});
