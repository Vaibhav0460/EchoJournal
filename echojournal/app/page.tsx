'use client';
import { useState } from 'react';
import { MessageSquare, Library, Settings, User } from 'lucide-react';
import CommandBar from '@/components/CommandBar';
import TimelineExplorer from '@/components/TimelineExplorer';
import RecentEchoes from '@/components/RecentEchoes';
import ProfileSettings from '@/components/ProfileSettings';

export default function Home() {
  const [activeView, setActiveView] = useState<'chat' | 'timeline' | 'profile'>('chat');
  const [refresh, setRefresh] = useState(0);

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
      </section>
    </main>
  );
}