'use client';
import { useState, useEffect, useMemo } from 'react';
import { writeTextFile, readTextFile, exists, mkdir } from '@tauri-apps/plugin-fs'; // Add mkdir
import { documentDir, join } from '@tauri-apps/api/path';
import { invoke } from '@tauri-apps/api/core';

export default function CommandBar({ onEntrySaved }: { onEntrySaved: () => void }) {
  const [input, setInput] = useState('');
  const [status, setStatus] = useState('Ready to echo...');
  const [debugDate, setDebugDate] = useState(new Date().toISOString().split('T')[0]);

  const { minDate, maxDate } = useMemo(() => {
    const today = new Date();
    const max = today.toISOString().split('T')[0];
    const minD = new Date();
    minD.setDate(today.getDate() - 7);
    const min = minD.toISOString().split('T')[0];
    
    return { minDate: min, maxDate: max };
  }, []);
  
  const handleJournalEntry = async (e: React.KeyboardEvent) => {
    if (e.key !== 'Enter' || !input.trim()) return;
    setStatus('Processing...');

    

    try {
      // 1. Guardrail Check
      const isLocked = await invoke<boolean>('is_date_locked', { targetDateStr: debugDate });
      if (isLocked) {
        setStatus("🚫 Guardrail Active: Date is locked.");
        return;
      }
  
      // 2. Content Preparation
      const isLiteral = input.startsWith('"') && input.endsWith('"');
      const finalContent = isLiteral ? input.slice(1, -1).trim() : await invoke<string>('refine_thought', { input });
  
      // 3. File Path Logic
      const [year, month, day] = debugDate.split('-');
      const fileName = `${year}-${month}.md`;
      const docsPath = await documentDir();
      const journalDir = await join(docsPath, 'EchoJournal');
      const filePath = await join(journalDir, fileName);

      const now = new Date();
      const todayDate = now.toISOString().split('T')[0];
      let timestampLabel: string;

      if (debugDate < todayDate) {
        timestampLabel = "Future Self";
      } else if (debugDate > todayDate) {
        timestampLabel = "Prescient Self";
      } else {
        timestampLabel = now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      }
      
      const newEntryLine = `- **${timestampLabel}**: ${finalContent}`;
  
      if (!(await exists(journalDir))) await mkdir(journalDir);
  
      // 4. THE PARSER (Treating Markdown as a Map)
      let rawContent = (await exists(filePath)) ? await readTextFile(filePath) : "";
      
      // Split content by headers, keeping the headers in the result
      const sections = rawContent.split(/(?=### \d{4}-\d{2}-\d{2})/).filter(s => s.trim() !== "");
      
      // Create a Map of Date -> Content
      const journalMap = new Map<string, string>();
      sections.forEach(section => {
        const match = section.match(/### (\d{4}-\d{2}-\d{2})/);
        if (match) {
          journalMap.set(match[1], section.replace(match[0], "").trim());
        }
      });
  
      // 5. UPDATE & SORT
      const existingEntries = journalMap.get(debugDate) || "";
      journalMap.set(debugDate, existingEntries + (existingEntries ? "\n" : "") + newEntryLine);
  
      // Sort dates chronologically
      const sortedDates = Array.from(journalMap.keys()).sort();
  
      // 6. SERIALIZE & OVERWRITE
      let finalMarkdown = "";
        sortedDates.forEach((date, index) => {
        // If it's not the first date in the file, add a separator
        const separator = index === 0 ? "" : "\n\n---\n\n";
        
        finalMarkdown += `${separator}### ${date}\n\n${journalMap.get(date)}\n\n`;
      });
  
      await writeTextFile(filePath, finalMarkdown.trim());
      onEntrySaved();
      setInput('');
      setStatus(`Echoed to ${debugDate}.`);
    } catch (err) {
      setStatus(`Error: ${err}`);
    }
  };

  return (
    <div className="w-full max-w-2xl space-y-4">
      <div className="flex gap-2 items-center bg-slate-900 border border-slate-700 p-2 rounded-xl focus-within:ring-2 focus-within:ring-blue-500 transition-all">
        {/* The "Time Travel" Debugger */}
        <input 
          type="date" 
          value={debugDate}
          min={minDate}
          max={maxDate}
          onChange={(e) => setDebugDate(e.target.value)}
          className="bg-slate-800 text-xs text-slate-400 p-2 rounded border-none focus:outline-none cursor-pointer hover:text-white"
        />
        
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleJournalEntry}
          placeholder="Type a thought..."
          className="flex-1 bg-transparent text-slate-100 p-2 focus:outline-none font-mono"
        />
      </div>
      <p className="text-xs text-slate-500 ml-2 italic">{status}</p>
    </div>
  );
}