import { describe, it, expect } from 'vitest';
import { contextMemory } from '../core/contextMemory';

describe('contextMemory', () => {
  it('stores and retrieves context', () => {
    contextMemory.saveContext('a', { foo: 1 });
    expect(contextMemory.getContext<any>('a')?.foo).toBe(1);
  });

  it('records prompt history', () => {
    contextMemory.recordPrompt('hi');
    expect(contextMemory.getHistory().length).toBeGreaterThan(0);
  });
});
