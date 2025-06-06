import { agentRegistry } from './core/agentRegistry';
import { orchestrator } from './core/orchestrator';
import { FrontendAgent } from './agents/frontend/frontendAgent';
import { BackendAgent } from './agents/backend/backendAgent';
import { DatabaseAgent } from './agents/database/databaseAgent';
import { DevOpsAgent } from './agents/devops/devopsAgent';
import { DocsAgent } from './agents/docs/docsAgent';

// Register all agents
try {
  agentRegistry.registerAgent('frontend', new FrontendAgent());
  agentRegistry.registerAgent('backend', new BackendAgent());
  agentRegistry.registerAgent('database', new DatabaseAgent());
  agentRegistry.registerAgent('devops', new DevOpsAgent());
  agentRegistry.registerAgent('docs', new DocsAgent());
} catch (err) {
  console.error('Failed to register agents:', err);
  process.exit(1);
}

async function main() {
  const prompt = 'Build a login page using React and store users in a database';
  const result = await orchestrator.receivePrompt(prompt);
  console.log('\n--- Result ---');
  console.log(result);
}

main().catch(err => console.error(err));
