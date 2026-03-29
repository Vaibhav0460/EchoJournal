'use client';
import { useEffect, useState } from 'react';
import { readTextFile, exists } from '@tauri-apps/plugin-fs';
import { documentDir, join } from '@tauri-apps/api/path';
import ReactMarkdown from 'react-markdown';

export default function Timeline({ refreshTrigger }: { refreshTrigger: number }) {
  const [days, setDays] = useState<{ date: string; content: string }[]>([]);
  const [loading, setLoading] = useState(true);

  const loadJournal = async () => {
    try {
      const now = new Date();
      const fileName = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}.md`;
      const docsPath = await documentDir();
      const filePath = await join(docsPath, 'EchoJournal', fileName);

      if (await exists(filePath)) {
        const raw = await readTextFile(filePath);
        // Split by date headers and filter out empty strings
        const sections = raw.split(/(?=### \d{4}-\d{2}-\d{2})/).filter(s => s.trim());
        
        const parsedDays = sections.map(section => {
          const lines = section.split('\n');
          const date = lines[0].replace('### ', '').trim();
          const content = lines.slice(1).join('\n').trim();
          return { date, content };
        }).reverse(); // Show newest dates at the top

        setDays(parsedDays);
      }
    } catch (err) {
      console.error("Failed to load timeline:", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadJournal();
  }, [refreshTrigger]); // Reload whenever a new entry is made

  if (loading) return <div className="text-slate-500 font-mono">Loading history...</div>;
  if (days.length === 0) return <div className="text-slate-500 font-mono">No echoes found for this month.</div>;

  return (
    <div className="w-full max-w-2xl space-y-8 pb-20">
      {days.map((day) => (
        <div key={day.date} className="relative pl-8 border-l border-slate-800">
          {/* Timeline Dot */}
          <div className="absolute -left-[5px] top-2 w-2 h-2 rounded-full bg-blue-500 shadow-[0_0_10px_rgba(59,130,246,0.5)]" />
          
          <h3 className="text-sm font-bold text-slate-400 mb-4 tracking-widest uppercase">
            {new Date(day.date).toLocaleDateString(undefined, { weekday: 'long', month: 'short', day: 'numeric' })}
          </h3>
          
          <div className="bg-slate-900/50 border border-slate-800/50 rounded-2xl p-6 backdrop-blur-sm shadow-xl">
            <div className="prose-strong:text-blue-400 prose-strong:font-mono prose-strong:text-xs">
                <ReactMarkdown>
                    {day.content}
                </ReactMarkdown>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}