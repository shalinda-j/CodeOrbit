import React from 'react';
import { contextMemory } from '../core/contextMemory';

/** Shows recent context entries for debugging */
export function SidePanel() {
  const entries = contextMemory.getHistory();
  return (
    <div className="side-panel border p-2 w-60 overflow-auto">
      <strong>Memory</strong>
      <ul>
        {entries.map((e, i) => <li key={i}>{e.prompt}</li>)}
      </ul>
    </div>
  );
}
