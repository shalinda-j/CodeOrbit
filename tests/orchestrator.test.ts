import { describe, it, expect } from 'vitest';
import { agentRegistry } from '../core/agentRegistry';
import { Orchestrator } from '../core/orchestrator';
import { FrontendAgent } from '../agents/frontend/frontendAgent';

describe('orchestrator', () => {
  it('routes prompt to frontend agent', async () => {
    agentRegistry.registerAgent('frontend', new FrontendAgent());
    const o = new Orchestrator();
    const res = await o.receivePrompt('make ui');
    expect(res).toMatch(/frontend/i);
  });
});
