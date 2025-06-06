import React from 'react';

export default function LandingPage() {
  return (
    <div className="min-h-screen flex flex-col items-center p-8">
      <img src="/assets/logo.svg" className="h-12 mb-4" alt="CodeOrbit" />
      <h1 className="text-2xl font-bold mb-2">CodeOrbit</h1>
      <p className="mb-6">AI-powered multi-agent code editor</p>
      <div className="space-x-4">
        <a href="#" className="btn">Download for Windows</a>
        <a href="#" className="btn">Download for macOS</a>
        <a href="#" className="btn">Download for Linux</a>
      </div>
    </div>
  );
}
