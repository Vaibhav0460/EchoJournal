'use client';
import { useState } from 'react';
import { writeTextFile, readTextFile, exists, mkdir } from '@tauri-apps/plugin-fs'; // Add mkdir
import { documentDir, join } from '@tauri-apps/api/path';
import { invoke } from '@tauri-apps/api/core';

export default function CommandBar() {
  const [input, setInput] = useState('');
  const [status, setStatus] = useState('Ready to echo...');

  const handleJournalEntry = async (e: React.KeyboardEvent) => {
    if (e.key !== 'Enter' || !input.trim()) return;

    setStatus('Processing...');
    
    const isLiteral = input.startsWith('"') && input.endsWith('"');
    let finalContent = "";

    if (isLiteral) {
    // .slice(1, -1) removes the first and last characters (the quotes)
    // .trim() handles any accidental spaces inside the quotes
    finalContent = input.slice(1, -1).trim(); 
    } else {
    try {
        setStatus('Processing...');
        finalContent = await invoke<string>('refine_thought', { input });
    } catch (err) {
        console.error("AI Error:", err);
        setStatus(`AI Error: ${err}`);
        finalContent = `[Manual]: ${input}`;
    }
    }

    try {
      const now = new Date();
      const fileName = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}.md`;
      const docsPath = await documentDir();
      const journalDir = await join(docsPath, 'EchoJournal');
      const filePath = await join(journalDir, fileName);

      // --- FIX: Create directory if it doesn't exist ---
      const dirExists = await exists(journalDir);
      if (!dirExists) {
        await mkdir(journalDir);
      }
      // ------------------------------------------------

      const timestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      const entry = `\n- **${timestamp}**: ${finalContent}`;

      let existingData = "";
      const fileExists = await exists(filePath);
      if (fileExists) {
        existingData = await readTextFile(filePath);
      }
      
      await writeTextFile(filePath, existingData + entry);
      
      setInput('');
      setStatus('Echoed to disk.');
    } catch (err) {
      console.error(err);
      setStatus('Write failed. Check console for details.');
    }
  };

  return (
    <div className="w-full max-w-2xl space-y-4">
      <input
        type="text"
        value={input}
        onChange={(e) => setInput(e.target.value)}
        onKeyDown={handleJournalEntry}
        placeholder="Type a thought..."
        className="w-full bg-slate-900 border border-slate-700 text-slate-100 p-4 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono"
      />
      <p className="text-xs text-slate-500 ml-2">{status}</p>
    </div>
  );
}