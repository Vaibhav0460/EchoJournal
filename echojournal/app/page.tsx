import CommandBar from '@/components/CommandBar';

export default function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center p-6 bg-slate-950 text-white">
      <div className="mb-12 text-center">
        <h1 className="text-5xl font-extrabold tracking-tighter bg-gradient-to-r from-blue-400 to-emerald-400 bg-clip-text text-transparent">
          EchoJournal
        </h1>
        <p className="text-slate-500 mt-2 font-mono text-sm">
          [ Local-First Metacognitive Framework ]
        </p>
      </div>
      
      <CommandBar />
    </main>
  );
}