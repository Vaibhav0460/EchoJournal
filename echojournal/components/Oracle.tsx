'use client';
import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { documentDir, join } from '@tauri-apps/api/path';
import { Send, Sparkles, Loader2, User } from 'lucide-react';
import ReactMarkdown from 'react-markdown';

interface Message {
  role: 'user' | 'assistant';
  content: string;
}

export default function Oracle() {
  const [query, setQuery] = useState('');
  const [messages, setMessages] = useState<Message[]>([]);
  const [loading, setLoading] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    scrollRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, loading]);

  const handleAsk = async () => {
    if (!query.trim() || loading) return;

    const userQuestion = query;
    setQuery('');
    setLoading(true);
    
    // Add User Question
    setMessages(prev => [...prev, { role: 'user', content: userQuestion }]);

    try {
      const docsPath = await documentDir();
      const journalPath = await join(docsPath, 'EchoJournal');
      const history = await invoke('get_all_entries', { journalPath });
      
      const response = await invoke<string>('ask_oracle', { 
        question: userQuestion, 
        journalData: JSON.stringify(history) 
      });

      // Start the reveal
      typeWriter(response);
    } catch (err) {
      setMessages(prev => [...prev, { role: 'assistant', content: `⚠️ The connection was severed: ${err}` }]);
      setLoading(false);
    }
  };

  const typeWriter = (text: string) => {
    let i = 0;
    let currentText = "";
    setLoading(false); // Stop showing the "Thinking" dots once typing starts

    const interval = setInterval(() => {
      currentText += text.charAt(i);
      setMessages(prev => {
        const last = prev[prev.length - 1];
        if (last?.role === 'assistant') {
          const newHistory = [...prev];
          newHistory[newHistory.length - 1] = { role: 'assistant', content: currentText };
          return newHistory;
        } else {
          return [...prev, { role: 'assistant', content: currentText }];
        }
      });
      i++;
      if (i >= text.length) clearInterval(interval);
    }, 10);
  };

  return (
    <div className="w-full max-w-3xl flex flex-col h-[85vh]">
      {/* 1. Added 'no-scrollbar' to hide the ugly bar */}
      <div className="flex-1 overflow-y-auto space-y-6 px-4 no-scrollbar pb-10">
        {messages.length === 0 && !loading && (
          <div className="h-full flex flex-col items-center justify-center opacity-20">
            <Sparkles size={48} className="text-purple-500 mb-4" />
            <p className="font-mono text-xs uppercase tracking-[0.4em]">Consulting the Echoes</p>
          </div>
        )}

        {messages.map((msg, idx) => (
          <div key={idx} className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'} animate-in fade-in slide-in-from-bottom-2`}>
            <div className={`max-w-[85%] p-6 rounded-2xl ${
              msg.role === 'user' 
              ? 'bg-blue-600/10 border border-blue-500/20 text-blue-100 rounded-tr-none' 
              : 'bg-slate-900 border border-purple-500/20 shadow-xl rounded-tl-none'
            }`}>
              <div className="flex items-center gap-2 mb-2 opacity-50">
                {msg.role === 'user' ? <User size={12} /> : <Sparkles size={12} className="text-purple-400" />}
                <span className="text-[10px] font-black uppercase tracking-tighter">
                  {msg.role === 'user' ? 'You' : 'Oracle'}
                </span>
              </div>
              <div className="prose prose-invert max-w-none text-sm leading-relaxed">
                <ReactMarkdown>{msg.content}</ReactMarkdown>
              </div>
            </div>
          </div>
        ))}

        {/* 2. The "Thinking" Animation Bubble */}
        {loading && (
          <div className="flex justify-start animate-in fade-in duration-300">
            <div className="bg-slate-900 border border-purple-500/20 p-4 rounded-2xl rounded-tl-none flex gap-1 items-center">
              <div className="w-1.5 h-1.5 bg-purple-500 rounded-full animate-bounce [animation-delay:-0.3s]"></div>
              <div className="w-1.5 h-1.5 bg-purple-500 rounded-full animate-bounce [animation-delay:-0.15s]"></div>
              <div className="w-1.5 h-1.5 bg-purple-500 rounded-full animate-bounce"></div>
            </div>
          </div>
        )}
        <div ref={scrollRef} />
      </div>

      {/* Input Bar */}
      <div className="mt-4 relative group px-4">
        <div className="absolute -inset-1 bg-gradient-to-r from-purple-600/10 to-blue-600/10 rounded-2xl blur opacity-75 group-focus-within:opacity-100 transition duration-1000"></div>
        <div className="relative flex items-center bg-slate-950 border border-slate-800 rounded-2xl p-2 shadow-2xl">
          <input 
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleAsk()}
            placeholder="Ask your past..."
            className="flex-1 bg-transparent py-4 px-4 focus:outline-none font-mono text-sm text-slate-100"
          />
          <button 
            onClick={handleAsk}
            disabled={loading}
            className="p-4 bg-purple-600 hover:bg-purple-500 text-white rounded-xl transition-all disabled:opacity-30"
          >
            <Send size={20} />
          </button>
        </div>
      </div>
    </div>
  );
}