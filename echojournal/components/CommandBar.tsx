'use client';
import { useState, useMemo } from 'react';
import { writeTextFile, readTextFile, exists, mkdir } from '@tauri-apps/plugin-fs';
import { documentDir, join } from '@tauri-apps/api/path';
import { invoke } from '@tauri-apps/api/core';
import { useSettings } from '@/hooks/useSettings';
import { loadSettings } from '@/lib/settings';

interface RefinedEntry {
  tag: string;
  content: string;
}

export default function CommandBar({ onEntrySaved }: { onEntrySaved: () => void }) {
  const [input, setInput] = useState('');
  const [status, setStatus] = useState('System ready...');
  const [debugDate, setDebugDate] = useState(new Date().toISOString().split('T')[0]);
  const { settings } = useSettings();

  const { minDate, maxDate } = useMemo(() => {
    const today = new Date();
    const minD = new Date();
    minD.setDate(today.getDate() - 7);
    return { 
      minDate: minD.toISOString().split('T')[0], 
      maxDate: today.toISOString().split('T')[0] 
    };
  }, []);

  const handleJournalEntry = async (e: React.KeyboardEvent) => {
    if (e.key !== 'Enter' || !input.trim()) return;
    setStatus('Refining (Offline)...');

    try {
      const currentSettings = await loadSettings();
      const isLocked = await invoke<boolean>('is_date_locked', { 
        date: debugDate
      });

      if (isLocked) {
        setStatus('Date is locked!');
        return;
      }

      // Call the backend refiner using the local LLM
      const refined = await invoke<RefinedEntry>('refine_thought', { 
        thought: input,
        useLocal: true 
      });

      const docsPath = await documentDir();
      const journalDir = await join(docsPath, 'EchoJournal');
      const filePath = await join(journalDir, 'journal.md');

      if (!(await exists(journalDir))) {
        await mkdir(journalDir, { recursive: true });
      }

      const rawContent = (await exists(filePath)) ? await readTextFile(filePath) : "";
      const newEntryLine = `(${refined.tag}) ${refined.content}`;

      // Organize entries by date
      const journalMap = new Map<string, string>();
      rawContent.split(/\n\n---\n\n/).forEach(s => {
        const date = s.match(/### (\d{4}-\d{2}-\d{2})/)?.[1];
        if (date) journalMap.set(date, s.split('\n').slice(1).join('\n').trim());
      });

      const existing = journalMap.get(debugDate) || "";
      journalMap.set(debugDate, existing + (existing ? "\n" : "") + newEntryLine);

      // Sort and rebuild the markdown file
      const sorted = Array.from(journalMap.keys()).sort();
      const finalMd = sorted.map((d, i) => `${i === 0 ? "" : "\n\n---\\n\\n"}### ${d}\n\n${journalMap.get(d)}`).join("");

      await writeTextFile(filePath, finalMd.trim());
      onEntrySaved();
      setInput('');
      setStatus(`Saved to #${refined.tag} (Offline)`);
    } catch (err) {
      setStatus(`Error: ${err}`);
      console.error(err);
    }
  };

  return (
    <div className="w-full max-w-2xl">
      <div className="flex gap-2 items-center bg-slate-900 border border-slate-700 p-2 rounded-2xl focus-within:ring-2 focus-within:ring-blue-500 shadow-xl transition-all">
        <input 
          type="date" 
          value={debugDate} 
          min={minDate} 
          max={maxDate} 
          onChange={(e) => setDebugDate(e.target.value)}
          className="bg-slate-800 text-xs text-slate-300 px-3 py-2 rounded-xl border border-slate-700 outline-none hover:border-slate-500 transition-colors cursor-pointer"
        />
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleJournalEntry}
          placeholder="Refine a thought locally..."
          className="flex-1 bg-transparent border-none outline-none text-slate-100 placeholder:text-slate-500 text-sm px-2"
        />
        <div className="px-3 py-1 bg-slate-800 rounded-lg">
          <span className="text-[10px] font-mono text-slate-400 uppercase tracking-tighter">
            Enter to Save
          </span>
        </div>
      </div>
      <p className="mt-2 text-center text-[10px] font-mono text-slate-500 uppercase tracking-widest animate-pulse">
        {status}
      </p>
    </div>
  );
}