<div align="center">

<!-- SUGGESTED IMAGE 1: A full-width hero screenshot of the main chat/command bar interface (the dark slate UI with the "Neural.Interface.Active" label and the command input bar glowing in focus). This is your app's most striking first impression. -->
![EchoJournal Hero](./docs/hero.png)

<br/>

<img src="./src-tauri/icons/128x128.png" alt="EchoJournal Logo" width="96"/>

# EchoJournal

### *Your thoughts. Your machine. Your truth.*

**A privacy-first, AI-powered journal that lives entirely on your device —  
no subscriptions, no cloud, no compromises.**

<br/>

[![Built with Tauri](https://img.shields.io/badge/Built%20with-Tauri%202-FFC131?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-CE422B?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Next.js](https://img.shields.io/badge/Next.js%2016-000000?style=for-the-badge&logo=next.js&logoColor=white)](https://nextjs.org)
[![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?style=for-the-badge&logo=typescript&logoColor=white)](https://www.typescriptlang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](./LICENSE)

<br/>

[✨ Features](#-features) · [🚀 Getting Started](#-getting-started) · [🧠 How It Works](#-how-it-works) · [🏗️ Architecture](#%EF%B8%8F-architecture) · [🗺️ Roadmap](#%EF%B8%8F-roadmap)

</div>

---

## The Problem with Modern Journaling

Every journaling app today makes the same quiet promise, then breaks it: *your thoughts are safe with us.* They live on someone else's server, indexed by someone else's algorithms, monetised in ways you'll never fully understand.

**EchoJournal refuses that bargain.**

Your journal entries are written to plain Markdown files on your own disk. The AI that reads and understands them runs on your own CPU or GPU. Nothing ever leaves your machine — not a single word, not a single vector.

---

## ✨ Features

### 🔒 Radically Local-First
Every byte of your journal stays on your device. Entries are stored as human-readable Markdown files in your `Documents/EchoJournal` folder — not locked in a proprietary database, not encrypted behind a paywall. You own your data, forever.

### 🧠 The Oracle — AI That Actually Knows You
Ask the Oracle anything about your past: *"When was I last feeling overwhelmed about work?"* or *"What has changed in my fitness routine over the last month?"* It retrieves semantically relevant entries using vector embeddings and responds with cited, grounded answers — no hallucinations about things you never wrote.

<!-- SUGGESTED IMAGE 2: A screenshot of the Oracle chat panel showing a multi-turn conversation, with the purple "Oracle" response bubbles and cited journal entries. This demonstrates the most wow-worthy feature. -->
![Oracle Chat](./docs/oracle.png)

### ⚡ Dual AI Modes — Online & Offline
| Mode | Backend | Internet Required |
|------|---------|-------------------|
| ☁️ **Cloud Oracle** | Google Gemini 1.5 Flash | Yes |
| 🖥️ **Local Oracle** | llama.cpp (any GGUF model) | **Never** |

Switch between modes with a single click. Take a flight, go off-grid, or simply choose not to trust a third-party — the Oracle works regardless.

### 🗂️ Smart Thought Refinement
Raw thoughts become polished journal entries automatically. Type a rough idea like *"rough day at standup, felt dismissed again"* and the AI refines it into a structured, tagged entry before it ever touches your disk.

### 📚 Timeline Explorer
Browse your entire journaling history month by month, rendered in beautiful Markdown. Every entry is timestamped, tagged, and searchable.

<!-- SUGGESTED IMAGE 3: A screenshot of the Timeline Explorer panel showing the month selector on the left and a rendered markdown journal page on the right, including the Export dropdown. This shows the polish of the data browsing experience. -->
![Timeline Explorer](./docs/timeline.png)

### 📤 Export to PDF & DOCX
Your memories deserve more than a folder of text files. Export any month's entries to a formatted PDF or Word document with a single click — ready to print, share with a therapist, or archive.

### 🔍 Semantic Vector Search
Powered by `fastembed` and the `AllMiniLML6V2` model, EchoJournal embeds every entry into a high-dimensional vector space stored in a local SQLite database. When you ask the Oracle a question, it finds the *most meaningfully similar* entries — not just keyword matches.

### 🎨 AI Personality Matrix
Define how the AI speaks to you for each tag category. Want your **Wellness** entries reflected back with empathy? Your **Finance** entries with cold calculation? Configure it in your Profile, and the Oracle adapts its tone accordingly.

### 🔔 Live File Watching
EchoJournal watches your journal folder in real time. Add an entry from another app, sync from a backup, or edit a file directly — the vector index updates automatically without you lifting a finger.

### 🔐 7-Day Entry Lock
Entries older than seven days are locked from editing. Your past is your past — immutable, honest, and protected from revisionism.

---

## 🚀 Getting Started

### Prerequisites

| Tool | Version |
|------|---------|
| [Node.js](https://nodejs.org) | ≥ 20.9.0 |
| [Rust](https://rustup.rs) | ≥ 1.77.2 |
| [Tauri CLI](https://tauri.app/start/) | v2 |

You will also need the platform-specific prerequisites for Tauri:  
→ [Linux](https://tauri.app/start/prerequisites/#linux) · [macOS](https://tauri.app/start/prerequisites/#macos) · [Windows](https://tauri.app/start/prerequisites/#windows)

---

### Installation

```bash
# 1. Clone the repository
git clone https://github.com/your-username/echojournal.git
cd echojournal

# 2. Install frontend dependencies
npm install

# 3. (Optional) Configure your Gemini API key for Cloud Oracle mode
#    Copy the example env file and add your key
cp .env.example .env
# Then edit .env and set: GEMINI_API_KEY=your_key_here

# 4. Launch in development mode
npm run tauri dev
```

> **First launch note:** On first run, EchoJournal will automatically download the `AllMiniLML6V2` embedding model (~25 MB) for local vector search. This is a one-time operation.

---

### Setting Up the Local Oracle (Offline AI)

To use the fully offline Local Oracle, you need a GGUF-format language model:

1. Download any compatible GGUF model — we recommend **Llama 3.2 3B Instruct (Q4_K_M)** for a great balance of speed and quality. Find models at [huggingface.co](https://huggingface.co/models?library=gguf).
2. Rename the file to `oracle-model.gguf`.
3. Place it in your app data directory:
   - **Linux:** `~/.local/share/com.echojournal.app/models/oracle-model.gguf`
   - **macOS:** `~/Library/Application Support/com.echojournal.app/models/oracle-model.gguf`
   - **Windows:** `%APPDATA%\com.echojournal.app\models\oracle-model.gguf`
4. Restart EchoJournal. The green **Local Oracle** indicator will appear.

---

### Building for Production

```bash
npm run tauri build
```

Your platform-native installer (`.dmg`, `.exe`, `.AppImage`) will appear in `src-tauri/target/release/bundle/`.

---

## 🧠 How It Works

EchoJournal's intelligence pipeline is a four-stage loop that runs entirely on your machine:

```
Your thought
     │
     ▼
┌─────────────────────────┐
│   1. REFINE              │  Raw text → polished entry + auto-tag
│   (Gemini or llama.cpp) │  via structured JSON prompt
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│   2. PERSIST             │  Appended to ~/Documents/EchoJournal/
│   (Plain Markdown)      │  YYYY-MM.md with date header
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│   3. EMBED               │  Entry → 384-dim float vector
│   (fastembed ONNX)      │  stored as BLOB in SQLite
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│   4. RETRIEVE & ANSWER   │  Query → cosine similarity search
│   (Oracle + LLM)        │  → top-k entries → grounded answer
└─────────────────────────┘
```

The entire pipeline — embedding model, vector database, and language model — runs as compiled native code via Rust. There is no Python runtime, no Docker container, no background server process beyond the app itself.

---

## 🏗️ Architecture

```
echojournal/
├── app/                        # Next.js frontend (static export)
│   ├── page.tsx                # Root shell & navigation state
│   └── globals.css             # Tailwind v4 theme
│
├── components/
│   ├── CommandBar.tsx          # Thought capture & refinement UI
│   ├── Oracle.tsx              # AI chat interface
│   ├── TimelineExplorer.tsx    # Month browser & export
│   ├── RecentEchoes.tsx        # Home feed of recent entries
│   └── ProfileSettings.tsx    # AI personality configuration
│
├── lib/
│   └── settings.ts             # Persistent user settings (JSON)
│
└── src-tauri/
    ├── src/
    │   ├── lib.rs              # All Tauri commands & app lifecycle
    │   ├── oracle_engine.rs    # llama.cpp inference wrapper
    │   ├── vector_store.rs     # fastembed + SQLite vector search
    │   ├── export.rs           # PDF (genpdf) & DOCX (docx-rs) export
    │   └── tags.rs             # Tag registry
    └── tauri.conf.json         # App configuration & permissions
```

### Key Technology Choices

| Layer | Technology | Why |
|-------|-----------|-----|
| Desktop shell | Tauri 2 | Rust-native, tiny binaries, no Electron overhead |
| Frontend | Next.js 16 + React 19 | Static export, instant HMR in dev |
| Styling | Tailwind CSS v4 | Zero-runtime, utility-first |
| Inference | llama-cpp-2 | Best-in-class GGUF support on CPU & GPU |
| Embeddings | fastembed + ONNX | 25 MB model, microsecond inference |
| Vector DB | SQLite (rusqlite) | Zero dependencies, single file, always available |
| Cloud AI | Google Gemini 1.5 Flash | Fastest, most affordable frontier model |
| Export | genpdf + docx-rs | Pure Rust, no system dependencies |


---

## 🗺️ Roadmap

- [ ] **Full-text search** across all entries with highlighted results
- [ ] **Mood tracking** — automatic sentiment analysis per entry with trend graphs
- [ ] **Streaks & habits** — consecutive journaling day counter
- [ ] **Voice input** — transcribe spoken thoughts directly via Whisper.cpp
- [ ] **Multi-vault support** — separate journals for work, personal, creative
- [ ] **E2E encrypted sync** — optional zero-knowledge backup to any S3-compatible storage
- [ ] **Mobile companion** (iOS / Android) — capture quick notes on the go, sync to desktop

---

## 🤝 Contributing

Contributions are warmly welcomed. Whether it's a bug fix, a new feature, or a documentation improvement — open an issue or submit a pull request.

```bash
# Fork, then clone your fork
git clone https://github.com/your-username/echojournal.git

# Create a feature branch
git checkout -b feat/your-amazing-feature

# Make your changes, then open a PR against main
```

Please follow the existing code style. Rust code should be `cargo fmt` clean. TypeScript should pass `npm run lint`.

---

## 📜 License

EchoJournal is open source software licensed under the [MIT License](./LICENSE).

---

<div align="center">

**Built with obsessive care for people who think deeply and value their privacy.**

*Your echoes belong to you.*

<br/>

⭐ **If EchoJournal resonates with you, star the repo — it means the world.**

</div>