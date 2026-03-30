'use client';
import { useState, useEffect } from 'react';
import { loadSettings, saveSettings, UserSettings } from '@/lib/settings';
import { Save, User, Sparkles } from 'lucide-react';

export default function ProfileSettings() {
  const [settings, setSettings] = useState<UserSettings | null>(null);
  const [status, setStatus] = useState('');

  useEffect(() => {
    loadSettings().then(setSettings);
  }, []);

  const handleSave = async () => {
    if (settings) {
      await saveSettings(settings);
      setStatus('Preferences locally encrypted and saved.');
      setTimeout(() => setStatus(''), 3000);
    }
  };

  if (!settings) return <div className="text-slate-500 font-mono">Loading Profile...</div>;

  return (
    <div className="w-full max-w-2xl space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <header className="flex justify-between items-center border-b border-slate-800 pb-6">
        <div>
          <h2 className="text-3xl font-black tracking-tighter">Profile</h2>
          <p className="text-slate-500 text-sm font-mono">Local Configuration Node</p>
        </div>
        <button 
          onClick={handleSave}
          className="flex items-center gap-2 bg-blue-600 hover:bg-blue-500 text-white px-4 py-2 rounded-xl transition-all font-bold text-sm shadow-lg shadow-blue-900/20"
        >
          <Save size={16} /> Save Changes
        </button>
      </header>

      {/* Basic Info */}
      <section className="bg-slate-900/40 p-6 rounded-2xl border border-slate-800">
        <div className="flex items-center gap-4 mb-4 text-blue-400">
          <User size={20} />
          <h3 className="font-bold uppercase tracking-widest text-xs">Identity</h3>
        </div>
        <input 
          value={settings.userName}
          onChange={(e) => setSettings({...settings, userName: e.target.value})}
          placeholder="Your Name"
          className="bg-slate-950 border border-slate-800 p-3 rounded-xl w-full focus:outline-none focus:ring-2 focus:ring-blue-500 text-slate-200"
        />
      </section>

      {/* AI Tone Matrix */}
      <section className="space-y-4">
        <div className="flex items-center gap-4 mb-2 text-purple-400">
          <Sparkles size={20} />
          <h3 className="font-bold uppercase tracking-widest text-xs">AI Personality Matrix</h3>
        </div>
        
        <div className="grid grid-cols-1 gap-3">
          {Object.entries(settings.tagTones).map(([tag, tone]) => (
            <div key={tag} className="group bg-slate-900/20 border border-slate-800 p-4 rounded-2xl hover:border-slate-700 transition-all">
              <label className="text-[10px] font-black text-slate-500 uppercase tracking-widest block mb-1">
                {tag} Tone
              </label>
              <input 
                value={tone}
                onChange={(e) => {
                  const newTones = { ...settings.tagTones, [tag]: e.target.value };
                  setSettings({ ...settings, tagTones: newTones });
                }}
                className="bg-transparent w-full text-slate-300 focus:outline-none font-mono text-sm italic"
              />
            </div>
          ))}
        </div>
      </section>

      {status && <p className="text-center text-blue-500 font-mono text-xs animate-pulse">{status}</p>}
    </div>
  );
}