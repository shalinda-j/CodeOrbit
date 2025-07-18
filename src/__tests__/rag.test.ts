import { RagAgent } from '../agents/rag/ragAgent';
import { agentRegistry } from '../core/agentRegistry';
import { orchestrator } from '../core/orchestrator';

describe('RagAgent', () => {
  it('should be able to run a multi-step task', async () => {
    agentRegistry.registerAgent(new RagAgent());
    const result = await orchestrator.receivePrompt(
      'rag:Create a new user;Update the user profile;Delete the user'
    );
    expect(result).toEqual('rag: Successfully executed 3 sub-tasks.');
  });
});
