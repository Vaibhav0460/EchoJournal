'use client';
import { useEffect, useState } from 'react';
import { readDir, readTextFile } from '@tauri-apps/plugin-fs';
import { documentDir, join } from '@tauri-apps/api/path';
import ReactMarkdown from 'react-markdown';
import { FileText, ChevronRight, Download, ChevronDown, FileJson, CheckCircle2 } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

export default function TimelineExplorer() {
  const [files, setFiles] = useState<string[]>([]);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [fileContent, setFileContent] = useState<string>("");
  const [isOpen, setIsOpen] = useState(false);
  const [status, setStatus] = useState('');

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
                <div className="relative inline-block text-left">
      <div>
        <button
          onClick={() => setIsOpen(!isOpen)}
          className="flex items-center gap-2 bg-slate-900 border border-slate-800 hover:border-blue-500/50 text-slate-300 px-4 py-2.5 rounded-xl transition-all text-xs font-bold tracking-widest shadow-xl"
        >
          <Download size={14} className="text-blue-400" />
          EXPORT 
          <ChevronDown size={14} className={`transition-transform duration-300 ${isOpen ? 'rotate-180' : ''}`} />
        </button>
      </div>

      {isOpen && (
        <>
          {/* Invisible backdrop to close dropdown when clicking outside */}
          <div className="fixed inset-0 z-10" onClick={() => setIsOpen(false)}></div>
          
          <div className="absolute right-0 mt-2 w-48 origin-top-right rounded-2xl bg-slate-900 border border-slate-800 shadow-2xl ring-1 ring-black ring-opacity-5 focus:outline-none z-20 animate-in fade-in zoom-in-95 duration-200">
            <div className="p-2 space-y-1">
              <button
                onClick={() => handleExport('docx')}
                className="flex w-full items-center gap-3 px-3 py-3 text-xs font-medium text-slate-300 hover:bg-blue-600/10 hover:text-blue-400 rounded-lg transition-colors"
              >
                <FileText size={16} />
                DOCX
              </button>
              
              <button
                onClick={() => handleExport('pdf')}
                className="flex w-full items-center gap-3 px-3 py-3 text-xs font-medium text-slate-300 hover:bg-red-600/10 hover:text-red-400 rounded-lg transition-colors"
              >
                <FileJson size={16} />
                PDF
              </button>
            </div>
          </div>
        </>
      )}

      {status && (
        <p className="absolute top-12 left-0 w-full text-[10px] text-blue-500 font-mono animate-pulse text-center">
          {status}
        </p>
      )}
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