/**
 * ContextMemory - A simple in-memory key-value store for managing agent context and state.
 * Implements a singleton pattern to ensure consistent state across the application.
 */
import fs from 'fs';
import { join } from 'path';
import dotenv from 'dotenv';
dotenv.config();

export type PersistenceType = 'memory' | 'file' | 'database';

export interface ContextMemoryOptions {
  maxEntriesPerAgent?: number;
  persistence?: PersistenceType;
  filePath?: string;
  dbPath?: string;
}

export class ContextMemory {
  private static instance: ContextMemory;
  private memory: Map<string, Map<string, any>>;
  private maxEntriesPerAgent: number;
  private persistence: PersistenceType;
  private filePath: string;
  private dbPath: string;
  private db: any;

  /**
   * Private constructor to enforce singleton pattern
   * @param maxEntriesPerAgent Maximum number of entries to keep per agent (for memory management)
   */
  private constructor(options: ContextMemoryOptions = {}) {
    this.memory = new Map();
    this.maxEntriesPerAgent = options.maxEntriesPerAgent ?? 100;
    this.persistence = options.persistence ?? 'memory';
    this.filePath = options.filePath ?? join(process.cwd(), 'context-memory.json');
    this.dbPath = options.dbPath ?? join(process.cwd(), 'context-memory.sqlite');
  }

  /**
   * Get the singleton instance of ContextMemory
   * @param maxEntriesPerAgent Optional: Set the max entries per agent (only used on first call)
   */
  public static getInstance(options?: number | ContextMemoryOptions): ContextMemory {
    if (!ContextMemory.instance) {
      if (typeof options === 'number') {
        ContextMemory.instance = new ContextMemory({ maxEntriesPerAgent: options });
      } else {
        ContextMemory.instance = new ContextMemory(options);
      }
    }
    return ContextMemory.instance;
  }

  /**
   * Save context data for a specific agent
   * @param agentId The ID of the agent
   * @param key The key to store the data under
   * @param value The value to store
   */
  public save(agentId: string, key: string, value: any): void {
    if (!agentId || typeof agentId !== 'string') {
      throw new Error('agentId must be a non-empty string');
    }
    
    let agentMemory = this.memory.get(agentId);
    
    // Initialize agent memory if it doesn't exist
    if (!agentMemory) {
      agentMemory = new Map();
      this.memory.set(agentId, agentMemory);
    }
    
    // Set the value
    agentMemory.set(key, value);
    
    // Enforce maximum entries if needed
    if (agentMemory.size > this.maxEntriesPerAgent) {
      // Get the first key from the map
      const firstKey = agentMemory.keys().next().value;
      if (firstKey !== undefined) {
        agentMemory.delete(firstKey);
      }
    }
    
    console.log(`[ContextMemory] Saved context for ${agentId}.${key}`);
  }

  /**
   * Get context data for a specific agent
   * @param agentId The ID of the agent
   * @param key The key to retrieve
   * @returns The stored value or undefined if not found
   */
  public get<T = any>(agentId: string, key: string): T | undefined {
    if (!agentId || typeof agentId !== 'string') {
      return undefined;
    }
    const agentMemory = this.memory.get(agentId);
    return agentMemory ? agentMemory.get(key) : undefined;
  }

  /**
   * Get all context data for a specific agent
   * @param agentId The ID of the agent
   * @returns An object containing all context data for the agent
   */
  public getAll(agentId: string): Record<string, any> {
    const agentMemory = this.memory.get(agentId);
    if (!agentMemory) return {};
    
    const result: Record<string, any> = {};
    for (const [key, value] of agentMemory.entries()) {
      result[key] = value;
    }
    return result;
  }

  /**
   * Delete a specific context entry for an agent
   * @param agentId The ID of the agent
   * @param key The key to delete
   * @returns true if the key existed and was deleted, false otherwise
   */
  public delete(agentId: string, key: string): boolean {
    return this.memory.get(agentId)?.delete(key) ?? false;
  }

  /**
   * Clear all context data for a specific agent
   * @param agentId The ID of the agent
   */
  public clearAgent(agentId: string): void {
    this.memory.delete(agentId);
    console.log(`[ContextMemory] Cleared all context for agent: ${agentId}`);
  }

  /**
   * Clear all context data for all agents
   */
  public clearAll(): void {
    this.memory.clear();
    console.log('[ContextMemory] Cleared all context data');
  }

  /**
   * Get the number of entries for a specific agent
   * @param agentId The ID of the agent
   * @returns The number of context entries for the agent, or 0 if none
   */
  public size(agentId: string): number {
    return this.memory.get(agentId)?.size ?? 0;
  }

  /**
   * Check if a specific key exists for an agent
   * @param agentId The ID of the agent
   * @param key The key to check
   * @returns true if the key exists, false otherwise
   */
  public has(agentId: string, key: string): boolean {
    return this.memory.get(agentId)?.has(key) ?? false;
  }

  /**
   * Get all agent IDs that have context stored
   * @returns Array of agent IDs
   */
  public getAgentIds(): string[] {
    return Array.from(this.memory.keys());
  }

  /**
   * Merge new context data with existing data for an agent
   * @param agentId The ID of the agent
   * @param key The key to merge data into
   * @param value The value to merge (must be an object)
   * @param deep Whether to perform a deep merge (default: true)
   */
  public merge(
    agentId: string,
    key: string,
    value: Record<string, any>,
    deep: boolean = true
  ): void {
    if (!this.memory.has(agentId)) {
      this.memory.set(agentId, new Map());
    }

    const agentMemory = this.memory.get(agentId)!;
    const existing = agentMemory.get(key) || {};
    
    const merged = deep ? this.deepMerge(existing, value) : { ...existing, ...value };
    agentMemory.set(key, merged);
    
    console.log(`[ContextMemory] Merged context for ${agentId}.${key}`);
  }

  /**
   * Helper method for deep merging objects
   * @private
   */
  private deepMerge(target: any, source: any): any {
    if (typeof target !== 'object' || target === null) {
      return source;
    }

    if (Array.isArray(target) && Array.isArray(source)) {
      return [...new Set([...target, ...source])];
    }

    const result = { ...target };
    
    for (const key in source) {
      if (source.hasOwnProperty(key)) {
        if (source[key] && typeof source[key] === 'object' && !Array.isArray(source[key])) {
          result[key] = this.deepMerge(
            target[key] || {},
            source[key]
          );
        } else {
          result[key] = source[key];
        }
      }
    }
    
    return result;
  }

  /** Load persisted context from file or database */
  public async load(): Promise<void> {
    if (this.persistence === 'file') {
      try {
        if (fs.existsSync(this.filePath)) {
          const raw = await fs.promises.readFile(this.filePath, 'utf-8');
          const data = JSON.parse(raw) as Record<string, Record<string, any>>;
          this.memory.clear();
          for (const [agentId, values] of Object.entries(data)) {
            this.memory.set(agentId, new Map(Object.entries(values)));
          }
          console.log(`[ContextMemory] Loaded data from ${this.filePath}`);
        }
      } catch (err) {
        console.error('[ContextMemory] Failed to load file persistence', err);
      }
    } else if (this.persistence === 'database') {
      try {
        const sqlite3 = await import('sqlite3');
        const dbModule: any = (sqlite3 as any).verbose ? (sqlite3 as any).verbose() : sqlite3;
        this.db = new dbModule.Database(this.dbPath);
        await new Promise((resolve, reject) => {
          this.db!.serialize(() => {
            this.db!.run(
              'CREATE TABLE IF NOT EXISTS context (agentId TEXT, key TEXT, value TEXT)',
              (err: Error) => {
                if (err) reject(err);
              }
            );
            this.db!.all('SELECT agentId, key, value FROM context', (err: Error, rows: any[]) => {
              if (err) {
                reject(err);
                return;
              }
              this.memory.clear();
              for (const row of rows) {
                let agentMem = this.memory.get(row.agentId);
                if (!agentMem) {
                  agentMem = new Map();
                  this.memory.set(row.agentId, agentMem);
                }
                agentMem.set(row.key, JSON.parse(row.value));
              }
              resolve(null);
            });
          });
        });
        console.log(`[ContextMemory] Loaded data from database ${this.dbPath}`);
      } catch (err) {
        console.error('[ContextMemory] Failed to load database persistence', err);
      }
    }
  }

  /** Save current context to file or database */
  public async save(): Promise<void> {
    if (this.persistence === 'file') {
      const data: Record<string, Record<string, any>> = {};
      for (const [agentId, map] of this.memory.entries()) {
        data[agentId] = Object.fromEntries(map.entries());
      }
      try {
        await fs.promises.writeFile(this.filePath, JSON.stringify(data, null, 2));
        console.log(`[ContextMemory] Saved data to ${this.filePath}`);
      } catch (err) {
        console.error('[ContextMemory] Failed to save file persistence', err);
      }
    } else if (this.persistence === 'database') {
      try {
        if (!this.db) {
          const sqlite3 = await import('sqlite3');
          const dbModule: any = (sqlite3 as any).verbose ? (sqlite3 as any).verbose() : sqlite3;
          this.db = new dbModule.Database(this.dbPath);
          this.db.run('CREATE TABLE IF NOT EXISTS context (agentId TEXT, key TEXT, value TEXT)');
        }
        await new Promise((resolve, reject) => {
          this.db!.serialize(() => {
            this.db!.run('DELETE FROM context');
            const stmt = this.db!.prepare('INSERT INTO context(agentId, key, value) VALUES (?, ?, ?)');
            for (const [agentId, map] of this.memory.entries()) {
              for (const [key, value] of map.entries()) {
                stmt.run(agentId, key, JSON.stringify(value));
              }
            }
            stmt.finalize((err: Error) => {
              if (err) reject(err);
              else resolve(null);
            });
          });
        });
        console.log(`[ContextMemory] Saved data to database ${this.dbPath}`);
      } catch (err) {
        console.error('[ContextMemory] Failed to save database persistence', err);
      }
    }
  }
}

// Export a singleton instance configured via environment variables
const persistence = process.env.CONTEXT_PERSISTENCE as PersistenceType | undefined;
const filePath = process.env.CONTEXT_FILE_PATH;
const dbPath = process.env.CONTEXT_DB_PATH;
export const contextMemory = ContextMemory.getInstance({
  persistence: persistence,
  filePath,
  dbPath
});
