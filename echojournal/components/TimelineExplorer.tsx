'use client';
import { useEffect, useState } from 'react';
import { readDir, readTextFile } from '@tauri-apps/plugin-fs';
import { documentDir, join } from '@tauri-apps/api/path';
import ReactMarkdown from 'react-markdown';
import { FileText, ChevronRight } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

export default function TimelineExplorer() {
  const [files, setFiles] = useState<string[]>([]);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [fileContent, setFileContent] = useState<string>("");

  useEffect(() => {
    const fetchFiles = async () => {
      try {
        const docsPath = await documentDir();
        const journalPath = await join(docsPath, 'EchoJournal');
        const entries = await readDir(journalPath);
        const mdFiles = entries
          .filter(e => e.name.endsWith('.md'))
          .map(e => e.name)
          .sort((a, b) => b.localeCompare(a)); // Newest month first
        setFiles(mdFiles);
      } catch (e) { console.error(e); }
    };
    fetchFiles();
  }, []);

  const openFile = async (name: string) => {
    const docsPath = await documentDir();
    const fullPath = await join(docsPath, 'EchoJournal', name);
    const content = await readTextFile(fullPath);
    setFileContent(content);
    setSelectedFile(name);
  };

  const handleExport = async (format: 'pdf' | 'docx') => {
    try {
      const docsPath = await documentDir();
      const journalPath = await join(docsPath, 'EchoJournal');
      const history = await invoke('get_all_entries', { journalPath });
      const result = await invoke<string>('export_journal', { 
        format, 
        entries: history 
      });
  
      console.log(result);
    } catch (err) {
      console.error("Export failed:", err);
    }
  };

  return (
    <div className="flex w-full h-full gap-8 animate-in slide-in-from-bottom-4 duration-500">
      {/* File List */}
      <div className="w-64 space-y-2 overflow-y-auto pr-4 border-r border-slate-800">
        <h3 className="text-xs font-bold text-slate-500 uppercase tracking-widest mb-4">Past Months</h3>
        {files.map(file => (
          <button
            key={file}
            onClick={() => openFile(file)}
            className={`w-full flex items-center gap-3 p-3 rounded-lg transition-all text-left text-sm ${selectedFile === file ? 'bg-blue-600/20 text-blue-400' : 'hover:bg-slate-900 text-slate-400'}`}
          >
            <FileText size={16} />
            {file.replace('.md', '')}
            <ChevronRight size={14} className="ml-auto opacity-50" />
          </button>
        ))}
      </div>

      {/* Markdown View */}
      
      <div className="flex-1 overflow-y-auto bg-slate-900/10 rounded-3xl p-10 border border-slate-800/30 shadow-2xl">
        {selectedFile ? (
            <div className="max-w-3xl mx-auto">
            <header className="mb-12 border-b border-slate-800 pb-6 flex justify-between items-end">
                <h2 className="text-3xl font-black tracking-tighter text-white">{selectedFile.replace('.md', '')}</h2>
                <span className="text-xs font-mono text-slate-500">Archive Node</span>
                <div className="flex flex-col gap-2">
                  <p className="text-[10px] text-slate-500 font-mono mb-2 uppercase tracking-widest">
                    Export Options
                  </p>
                  <div className="flex gap-3">
                    {/* DOCX is the safest bet for a demo - no font dependencies! */}
                    <button 
                      onClick={() => handleExport('docx')} 
                      className="flex-1 bg-blue-600/10 hover:bg-blue-600/20 text-blue-400 border border-blue-500/20 py-3 rounded-xl transition-all text-xs font-bold"
                    >
                      GENERATE DOCX
                    </button>

                    <button 
                      onClick={() => handleExport('pdf')} 
                      className="flex-1 bg-slate-800/50 hover:bg-slate-800 text-slate-400 border border-slate-700 py-3 rounded-xl transition-all text-xs font-bold"
                    >
                      GENERATE PDF
                    </button>
                  </div>
                  <p className="text-[9px] text-slate-600 italic">
                    * PDF requires font embedding; DOCX is recommended for editing.
                  </p>
                </div>
            </header>

            <div className="prose prose-invert max-w-none 
                prose-hr:border-slate-800 prose-hr:my-16
                prose-h3:text-blue-400 prose-h3:text-2xl prose-h3:font-black prose-h3:tracking-tighter
                prose-h3:mt-0 prose-h3:mb-8
                prose-ul:space-y-4
                prose-li:text-slate-300 prose-li:leading-relaxed
                prose-strong:text-blue-300 prose-strong:bg-blue-900/30 prose-strong:px-2 prose-strong:py-1 prose-strong:rounded-lg"
                >
                <ReactMarkdown>{fileContent}</ReactMarkdown>
            </div>
            </div>
        ) : (
            <div className="h-full flex flex-col items-center justify-center text-slate-700 font-mono text-sm uppercase tracking-widest">
            Select a temporal slice
            </div>
        )}
        </div>
    </div>
  );
}