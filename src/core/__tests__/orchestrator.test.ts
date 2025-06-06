import { agentRegistry } from "../agentRegistry";
import { orchestrator } from "../orchestrator";
import type { IAgent, AgentResult } from "../Agent";

describe("Orchestrator prompt sanitization", () => {
  class EchoAgent implements IAgent {
    id = "frontend";
    name = "Echo";
    description = "echo";
    capabilities: string[] = [];
    async execute(input: string): Promise<AgentResult> {
      return { success: true, output: input };
    }
  }

  beforeEach(() => {
    agentRegistry.clear();
    agentRegistry.registerAgent(new EchoAgent());
    orchestrator.clearQueue();
  });

  test("removes script tags from prompt", async () => {
    const malicious = "<script>alert(1)</script> generate ui";
    const result = await orchestrator.receivePrompt(malicious);
    expect(result.output).toBe("generate ui");
  });

  test("strips command injection characters", async () => {
    const malicious = "list files && rm -rf /";
    const result = await orchestrator.receivePrompt(malicious);
    expect(result.output).toBe("list files rm -rf /");
  });
});
