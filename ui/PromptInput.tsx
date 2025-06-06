import React, { useState } from 'react';
import { sendPrompt } from '../core/orchestrator';

/** PromptInput provides a text box for the user to send prompts to CodeOrbit */
export function PromptInput({ onResponse }: { onResponse: (resp: string) => void }) {
  const [prompt, setPrompt] = useState('');

  async function submit() {
    const clean = prompt.replace(/[^\w\s.,!?]/g, '');
    if (!clean.trim()) return;
    setPrompt('');
    const resp = await sendPrompt(clean);
    onResponse(resp);
  }

  return (
    <div className="prompt-input flex">
      <input
        className="flex-grow border p-2 rounded"
        value={prompt}
        onChange={e => setPrompt(e.target.value)}
        onKeyDown={e => { if (e.key === 'Enter') submit(); }}
        placeholder="Ask CodeOrbit..."
      />
      <button className="ml-2 px-3" onClick={submit}>Send</button>
    </div>
  );
}
