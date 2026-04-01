'use client';
import { useState, useEffect } from 'react';
import { MessageSquare, Library, Settings, User, Sparkles, Loader2 } from 'lucide-react';
import CommandBar from '@/components/CommandBar';
import TimelineExplorer from '@/components/TimelineExplorer';
import RecentEchoes from '@/components/RecentEchoes';
import ProfileSettings from '@/components/ProfileSettings';
import { invoke } from '@tauri-apps/api/core';
import { documentDir, join } from '@tauri-apps/api/path';
import Oracle from '@/components/Oracle';

interface RawEntry {
  date: string;
  time: string;
  text: string;
  tag: string;
}

export default function Home() {
  const [activeView, setActiveView] = useState<'chat' | 'timeline' | 'profile' | 'oracle'>('chat');
  const [refresh, setRefresh] = useState(0);
  const [syncStatus, setSyncStatus] = useState<string | null>(null);
  
  const runAggregatorTest = async () => {
    try {
      const docsPath = await documentDir();
      const journalPath = await join(docsPath, 'EchoJournal');
      
      console.log("Fetching from:", journalPath);
      
      // Call the Rust command
      const history = await invoke('get_all_entries', { journalPath });
      
      console.log("✅ Aggregator Success!");
      console.table(history); // This prints a nice table in the console
    } catch (error) {
      console.error("❌ Aggregator Failed:", error);
    }
  };

  // --- STARTUP SYNC TRIGGER ---
  useEffect(() => {
    const runStartupSync = async () => {
      try {
        setSyncStatus("Syncing...");
        const docsPath = await documentDir();
        const journalPath = await join(docsPath, 'EchoJournal');
        
        // Triggers your Rust 'sync_vectors' function
        const result = await invoke<string>('sync_vectors', { journalPath });
        
        console.log("Background Sync:", result);
        setSyncStatus(null); // Clear status on success
      } catch (error) {
        console.error("Startup Sync Failed:", error);
        setSyncStatus("Sync Failed");
      }
    };

    runStartupSync();
  }, []); // Empty dependency array ensures this runs once on start/refresh

  return (
    <main className="flex h-screen bg-slate-950 text-slate-100 overflow-hidden">
      {/* --- SIDEBAR --- */}
      <nav className="w-20 border-r border-slate-800 flex flex-col items-center py-8 gap-8 bg-slate-900/30">
        <div className="w-10 h-10 bg-blue-600 rounded-xl flex items-center justify-center font-black text-xl mb-4">E</div>
        
        <button 
          onClick={() => setActiveView('chat')}
          className={`p-3 rounded-xl transition-all ${activeView === 'chat' ? 'bg-blue-600/20 text-blue-400' : 'text-slate-500 hover:text-white'}`}
        >
          <MessageSquare size={24} />
        </button>

        <button 
          onClick={() => setActiveView('timeline')}
          className={`p-3 rounded-xl transition-all ${activeView === 'timeline' ? 'bg-blue-600/20 text-blue-400' : 'text-slate-500 hover:text-white'}`}
        >
          <Library size={24} />
        </button>
        <button 
          onClick={() => setActiveView('oracle')}
          className={`p-3 rounded-xl transition-all ${activeView === 'oracle' ? 'bg-purple-600/20 text-purple-400' : 'text-slate-500 hover:text-white'}`}
        >
          <Sparkles size={24} />
        </button>

        <button 
          onClick={() => setActiveView('profile')}
          className={`p-3 rounded-xl transition-all ${activeView === 'profile' ? 'bg-blue-600/20 text-blue-400' : 'text-slate-500 hover:text-white'}`}
        >
          <User size={24} />
        </button>
      </nav>

      {/* --- MAIN CONTENT --- */}
      <section className="flex-1 flex flex-col items-center justify-center p-8 relative overflow-y-auto">
      {activeView === 'chat' && (
          <div className="w-full max-w-2xl flex flex-col items-center animate-in fade-in zoom-in duration-300">
             <h2 className="text-xs font-mono mb-12 text-slate-500 uppercase tracking-[0.4em]">Neural.Interface.Active</h2>
             <CommandBar onEntrySaved={() => setRefresh(r => r + 1)} />
             <RecentEchoes refreshTrigger={refresh} />
          </div>
        )}

        {activeView === 'timeline' && <TimelineExplorer />}

        {activeView === 'profile' && <ProfileSettings />}

        {activeView === 'oracle' && <Oracle />}
      </section>
    </main>
  );
}