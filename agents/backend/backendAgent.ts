import { Agent } from '../base';

/** A dummy backend agent handling server-side tasks */
export class BackendAgent implements Agent {
  readonly name = 'backend';

  async run(task: string): Promise<string> {
    console.log(`[BackendAgent] processing: ${task}`);
    try {
      await delay(100);
      return `Backend response for: ${task}`;
    } catch (err) {
      console.error('BackendAgent error:', err);
      return 'backend error';
    }
  }

  getCapabilities(): string[] {
    return ['api-design', 'business-logic'];
  }
}

async function delay(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}
