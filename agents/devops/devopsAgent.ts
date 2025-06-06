import { Agent } from '../base';

/** A dummy DevOps agent managing deployment and infrastructure tasks */
export class DevOpsAgent implements Agent {
  readonly name = 'devops';

  async run(task: string): Promise<string> {
    console.log(`[DevOpsAgent] processing: ${task}`);
    try {
      await delay(100);
      return `DevOps response for: ${task}`;
    } catch (err) {
      console.error('DevOpsAgent error:', err);
      throw err;
    }
  }

  getCapabilities(): string[] {
    return ['deployment', 'infrastructure'];
  }
}

async function delay(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}
