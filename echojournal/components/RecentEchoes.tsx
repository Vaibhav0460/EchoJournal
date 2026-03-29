'use client';
import { useEffect, useState } from 'react';
import { readTextFile, exists } from '@tauri-apps/plugin-fs';
import { documentDir, join } from '@tauri-apps/api/path';
import ReactMarkdown from 'react-markdown';

export default function RecentEchoes({ refreshTrigger }: { refreshTrigger: number }) {
  const [entries, setEntries] = useState<string[]>([]);

  useEffect(() => {
    const loadRecent = async () => {
      try {
        const now = new Date();
        const fileName = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}.md`;
        const docsPath = await documentDir();
        const filePath = await join(docsPath, 'EchoJournal', fileName);

        if (await exists(filePath)) {
          const raw = await readTextFile(filePath);
          // Split by bullets, filter empty, take last 5
          const bullets = raw.split('\n- ').filter(b => b.trim() && !b.startsWith('###'));
          setEntries(bullets.slice(-5).map(b => b.replace(/^- /, '')));
        }
      } catch (e) { console.error(e); }
    };
    loadRecent();
  }, [refreshTrigger]);

  if (entries.length === 0) return null;

  return (
    <div className="w-full mt-12 space-y-4 animate-in fade-in slide-in-from-bottom-4 duration-1000">
      <h3 className="text-[10px] font-bold text-slate-600 uppercase tracking-[0.3em] mb-6 text-center">Recent Echoes</h3>
      {entries.map((entry, i) => (
        <div 
          key={i} 
          className="bg-slate-900/40 border border-slate-800/50 p-4 rounded-xl backdrop-blur-sm hover:border-blue-500/30 transition-colors group"
        >
          <div className="prose prose-invert prose-sm max-w-none 
            prose-strong:text-blue-400 prose-strong:font-mono prose-strong:text-xs prose-strong:bg-blue-900/20 prose-strong:px-2 prose-strong:py-0.5 prose-strong:rounded-md"
          >
            <ReactMarkdown>{`- ${entry}`}</ReactMarkdown>
          </div>
        </div>
      ))}
    </div>
  );
}