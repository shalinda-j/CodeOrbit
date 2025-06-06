import { Agent } from '../base';

/** A dummy documentation agent */
export class DocsAgent implements Agent {
  readonly name = 'docs';

  async run(task: string): Promise<string> {
    console.log(`[DocsAgent] processing: ${task}`);
    try {
      await delay(100);
      return `Docs response for: ${task}`;
    } catch (err) {
      console.error('DocsAgent error:', err);
      throw err;
    }
  }

  getCapabilities(): string[] {
    return ['documentation'];
  }
}

async function delay(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}
