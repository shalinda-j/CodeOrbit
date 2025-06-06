import React, { useState } from 'react';
import { createRoot } from 'react-dom/client';
import { PromptInput } from './PromptInput';
import { OutputDisplay } from './OutputDisplay';
import { AgentSelector } from './AgentSelector';
import { SidePanel } from './SidePanel';

export function App() {
  const [output, setOutput] = useState('');
  return (
    <div className="flex">
      <SidePanel />
      <div className="flex-1 p-4">
        <AgentSelector />
        <PromptInput onResponse={setOutput} />
        <OutputDisplay text={output} />
      </div>
    </div>
  );
}

declare const document: any;
const root = createRoot(document.getElementById('root'));
root.render(<App />);
