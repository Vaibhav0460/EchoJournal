'use client';
import { useState, useMemo } from 'react';
import { writeTextFile, readTextFile, exists, mkdir } from '@tauri-apps/plugin-fs';
import { documentDir, join } from '@tauri-apps/api/path';
import { invoke } from '@tauri-apps/api/core';
import { useSettings } from '@/hooks/useSettings';
import { loadSettings } from '@/lib/settings';

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
    setStatus('Analyzing...');

    try {
      const currentSettings = await loadSettings();
      const isLocked = await invoke<boolean>('is_date_locked', { targetDateStr: debugDate });
      if (isLocked) {
        setStatus("🚫 Locked");
        return;
      }

      // AI returns both the text and the auto-detected tag
      const refined = await invoke<{ text: string, tag: string }>('refine_thought', { 
        input, 
        settingsJson: JSON.stringify(currentSettings) 
      });

      const [year, month] = debugDate.split('-');
      const fileName = `${year}-${month}.md`;
      const docsPath = await documentDir();
      const filePath = await join(docsPath, 'EchoJournal', fileName);

      if (!(await exists(await join(docsPath, 'EchoJournal')))) {
        await mkdir(await join(docsPath, 'EchoJournal'));
      }

      const timestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      // Saving with the hashtag for easy searching later
      const newEntryLine = `- **${timestamp}**: ${refined.text} #${refined.tag}`;

      let rawContent = (await exists(filePath)) ? await readTextFile(filePath) : "";
      const sections = rawContent.split(/(?=### \d{4}-\d{2}-\d{2})/).filter(s => s.trim());
      const journalMap = new Map<string, string>();
      
      sections.forEach(s => {
        const date = s.match(/### (\d{4}-\d{2}-\d{2})/)?.[1];
        if (date) journalMap.set(date, s.split('\n').slice(1).join('\n').trim());
      });

      const existing = journalMap.get(debugDate) || "";
      journalMap.set(debugDate, existing + (existing ? "\n" : "") + newEntryLine);

      const sorted = Array.from(journalMap.keys()).sort();
      const finalMd = sorted.map((d, i) => `${i === 0 ? "" : "\n\n---\n\n"}### ${d}\n\n${journalMap.get(d)}`).join("");

      await writeTextFile(filePath, finalMd.trim());
      onEntrySaved();
      setInput('');
      setStatus(`Saved to #${refined.tag}`);
    } catch (err) {
      setStatus(`Error: ${err}`);
    }
  };

  return (
    <div className="w-full max-w-2xl">
      <div className="flex gap-2 items-center bg-slate-900 border border-slate-700 p-2 rounded-2xl focus-within:ring-2 focus-within:ring-blue-500">
        <input type="date" value={debugDate} min={minDate} max={maxDate} 
          onChange={(e) => setDebugDate(e.target.value)}
          className="bg-slate-800 text-xs text-slate-400 p-2 rounded-lg cursor-pointer" />
        <input type="text" value={input} onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleJournalEntry} placeholder="Echo a thought..."
          className="flex-1 bg-transparent p-2 focus:outline-none font-mono text-slate-100" />
      </div>
      <p className="text-[10px] text-slate-600 mt-2 ml-2 uppercase tracking-widest">{status}</p>
    </div>
  );
}