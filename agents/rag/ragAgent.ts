import { Agent } from "../base";

export class RagAgent implements Agent {
  public readonly name = "rag";

  public async run(task: string): Promise<string> {
    console.log("RAG agent received task:", task);
    const subTasks = task.split(";").map((t) => t.trim());
    console.log("Parsed sub-tasks:", subTasks);

    // In a real implementation, we would use a RAG model to process the tasks.
    // For now, we'll just simulate the execution.
    for (const subTask of subTasks) {
      console.log(`Executing sub-task: ${subTask}`);
    }

    return `Successfully executed ${subTasks.length} sub-tasks.`;
  }

  public getCapabilities(): string[] {
    return ["multi-task-execution", "rag-based-processing"];
  }
}
