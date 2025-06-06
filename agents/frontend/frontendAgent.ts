import { Agent } from '../base';

/** A dummy frontend agent simulating UI-related tasks */
export class FrontendAgent implements Agent {
  readonly name = 'frontend';

  async run(task: string): Promise<string> {
    // TODO: Replace with real UI generation logic
    console.log(`[FrontendAgent] processing: ${task}`);
    try {
      await delay(100); // simulate async work
      return `Frontend response for: ${task}`;
    } catch (err) {
      console.error('FrontendAgent error:', err);
      return 'frontend error';
    }
  }

  getCapabilities(): string[] {
    return ['ui-planning', 'component-generation'];
  }
}

async function delay(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}
