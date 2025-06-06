import React from 'react';

/** Displays orchestrator responses in a scrollable panel */
export function OutputDisplay({ text }: { text: string }) {
  return (
    <pre className="output-display p-2 bg-gray-100 h-64 overflow-auto border rounded">
      {text}
    </pre>
  );
}
