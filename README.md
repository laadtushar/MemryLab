# Memory Palace

<!-- Badges: Top Row - Project Identity -->
<p align="center">
  <a href="https://github.com/laadtushar/MemPalace/actions/workflows/ci.yml"><img src="https://github.com/laadtushar/MemPalace/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <img src="https://img.shields.io/badge/version-0.1.0-8b5cf6?style=flat-square" alt="Version" />
  <img src="https://img.shields.io/badge/status-MVP-yellow?style=flat-square" alt="Status" />
  <img src="https://img.shields.io/badge/license-MIT-green?style=flat-square" alt="License" />
  <img src="https://img.shields.io/badge/PRs-welcome-brightgreen?style=flat-square" alt="PRs Welcome" />
</p>

<!-- Badges: Tech Stack -->
<p align="center">
  <img src="https://img.shields.io/badge/Tauri-2.0-24C8D8?style=flat-square&logo=tauri&logoColor=white" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/Rust-2021-CE422B?style=flat-square&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/React-19-61DAFB?style=flat-square&logo=react&logoColor=black" alt="React 19" />
  <img src="https://img.shields.io/badge/TypeScript-6.0-3178C6?style=flat-square&logo=typescript&logoColor=white" alt="TypeScript" />
  <img src="https://img.shields.io/badge/Tailwind_CSS-4.0-06B6D4?style=flat-square&logo=tailwindcss&logoColor=white" alt="Tailwind CSS" />
  <img src="https://img.shields.io/badge/D3.js-7-F9A03C?style=flat-square&logo=d3dotjs&logoColor=white" alt="D3.js" />
  <img src="https://img.shields.io/badge/SQLite-FTS5-003B57?style=flat-square&logo=sqlite&logoColor=white" alt="SQLite" />
  <img src="https://img.shields.io/badge/Zustand-5-443E38?style=flat-square" alt="Zustand" />
  <img src="https://img.shields.io/badge/Vite-8-646CFF?style=flat-square&logo=vite&logoColor=white" alt="Vite" />
</p>

<!-- Badges: AI & Privacy -->
<p align="center">
  <img src="https://img.shields.io/badge/LLM-Ollama_Compatible-000000?style=flat-square&logo=ollama" alt="Ollama" />
  <img src="https://img.shields.io/badge/LLM-Claude_API-cc785c?style=flat-square" alt="Claude" />
  <img src="https://img.shields.io/badge/Privacy-Local_First-blueviolet?style=flat-square" alt="Privacy" />
  <img src="https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=flat-square" alt="Platform" />
  <img src="https://img.shields.io/badge/Architecture-Hexagonal_(Ports_%26_Adapters)-informational?style=flat-square" alt="Architecture" />
  <img src="https://img.shields.io/badge/Tests-49_passing-brightgreen?style=flat-square" alt="Tests" />
</p>

> *A searchable, visual timeline of how your thinking evolved.*

**Memory Palace** is a privacy-first native desktop application that ingests your digital footprint — journal entries, chat exports, social media archives, notes — and constructs a searchable, visual timeline of how your thinking, interests, and emotional patterns have evolved over time.

It doesn't merely catalog events; it surfaces the **meta-narrative of personal growth and change**.

---

## Why Memory Palace?

People generate vast amounts of text about themselves but have **no tool to understand the arc of who they're becoming**. Memory Palace makes the invisible pattern visible.

- **"How has my view on remote work changed?"** — Evolution queries across years of writing
- **"When did I stop mentioning anxiety?"** — Temporal pattern detection
- **"What contradicts what I believed 3 years ago?"** — Belief contradiction surfacing
- **"Show me the themes of summer 2022"** — Time-windowed topic analysis

---

## Features

### Core
| Feature | Description |
|---------|-------------|
| **Multi-Source Import** | Obsidian vaults, Markdown/text files, Day One journals (WhatsApp, Telegram, Twitter in v0.2+) |
| **Interactive Timeline** | D3.js-powered zoomable bar chart — see document density and activity patterns over time |
| **Keyword Search** | BM25-ranked full-text search via SQLite FTS5 |
| **Semantic Search** | Vector cosine similarity search using local embeddings (Ollama nomic-embed-text) |
| **Hybrid Search** | Reciprocal Rank Fusion (k=60) combining keyword + semantic for best-of-both results |
| **Ask Your Memory** | RAG pipeline — natural language questions grounded in your writing, with source citations |
| **Analysis Engine** | Theme extraction, belief tracking, sentiment analysis, insight generation |
| **Memory Browser** | Browse, filter, and delete extracted facts, beliefs, preferences, and self-descriptions |
| **Insight Feed** | AI-generated observations about personal evolution and patterns |

### Privacy & Security
- **100% local by default** — All data stored and processed on-device
- **Zero telemetry** — No analytics, no phone-home, no cloud dependency
- **Local LLM support** — Full functionality with Ollama (Llama 3.1, Mistral, etc.)
- **Optional cloud LLM** — Bring your own Claude API key for enhanced analysis
- **Encrypted storage** — SQLite with SQLCipher encryption support

### Architecture
- **Hexagonal (Ports & Adapters)** — Swap any component without touching business logic
- **9 port interfaces** — Document, Vector, Graph, LLM, Embedding, Memory, PageIndex, Timeline, Analysis
- **Polyglot persistence** — SQLite for documents/graphs/FTS, vector store for embeddings
- **49 unit & integration tests** across all modules

---

## Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **App Shell** | [Tauri 2.0](https://v2.tauri.app/) | Native desktop (Rust backend + web frontend) |
| **Backend** | Rust (2021 edition) | Memory safety, performance, async runtime |
| **Frontend** | React 19 + TypeScript 6 | Component-based UI with type safety |
| **Styling** | Tailwind CSS 4 + Lucide Icons | Dark theme, utility-first, zero runtime |
| **State** | Zustand 5 | Lightweight TypeScript-native state management |
| **Visualization** | D3.js 7 | Interactive timeline with zoom/pan controls |
| **Database** | SQLite (rusqlite, bundled) | Documents, chunks, entities, memory facts, config |
| **Full-Text Search** | SQLite FTS5 | BM25-ranked keyword search with auto-sync triggers |
| **Vector Search** | SQLite + cosine similarity | Semantic search (upgradable to LanceDB) |
| **Graph Storage** | SQLite adjacency model | Entity relationships with recursive CTE traversal |
| **Local LLM** | [Ollama](https://ollama.com/) | Embeddings (nomic-embed-text), completions (Llama 3.1) |
| **Cloud LLM** | Claude API (Anthropic) | Optional enhanced analysis and narrative generation |
| **Build** | Vite 8 + Cargo | Fast dev builds, cross-platform packaging |
| **CI/CD** | GitHub Actions | Cross-platform build + test (Ubuntu, Windows, macOS) |

---

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [Node.js](https://nodejs.org/) (18+)
- [Ollama](https://ollama.com/) (for local LLM features)

### Setup

```bash
# Clone the repo
git clone https://github.com/laadtushar/MemPalace.git
cd MemPalace

# Install frontend dependencies
npm install

# Pull recommended Ollama models
ollama pull nomic-embed-text
ollama pull llama3.1:8b

# Run in development mode
cargo tauri dev

# Build for production
cargo tauri build
```

### Import Your Data

1. Open the app and click **Import** in the sidebar
2. Select a source type (Obsidian Vault, Markdown folder, or Day One JSON)
3. Pick the file/folder — watch the progress bar
4. Your documents are now searchable!

### Search & Ask

- **Keyword**: Type any word to find exact matches (BM25 ranking)
- **Semantic**: Find content by meaning ("times I felt uncertain")
- **Hybrid**: Best of both — combines keyword + semantic via RRF
- **Ask**: Natural language questions with RAG — "How has my view on career changed?"

---

## UI Views

| View | Description |
|------|-------------|
| **Timeline** | D3.js interactive bar chart with zoom/pan, monthly document counts, activity table |
| **Search** | Multi-mode search (keyword/semantic/hybrid), result cards, document viewer |
| **Ask** | Chat-style RAG interface with suggested questions and source citations |
| **Insights** | AI-generated observations with analysis trigger button |
| **Import** | Guided wizard for Obsidian, Markdown, and Day One imports with progress tracking |
| **Memory** | Filterable browser of extracted facts, beliefs, preferences — with delete controls |
| **Settings** | Ollama connection test, model list, app statistics dashboard |

---

## Architecture

Memory Palace follows a strict **hexagonal architecture**. The core domain has zero knowledge of specific databases, LLM providers, or UI frameworks.

```
┌─────────────────────────────────────────────────────────┐
│                     React Frontend                       │
│  Timeline │ Search │ Ask │ Insights │ Memory │ Settings  │
├───────────────────────┬─────────────────────────────────┤
│    Tauri Commands (15) │      Events (import progress)   │
├───────────────────────┴─────────────────────────────────┤
│                                                          │
│   ┌──────────────────────────────────────────────────┐  │
│   │                Domain (Pure Logic)                │  │
│   │  Models: Document, Chunk, Entity, Theme, Insight │  │
│   │  Ports:  IDocumentStore, IVectorStore,            │  │
│   │          ILLMProvider, IMemoryStore, IPageIndex,  │  │
│   │          IGraphStore, ITimelineStore,             │  │
│   │          IEmbeddingProvider, IAnalysisStage       │  │
│   └──────────────────────────────────────────────────┘  │
│          ▲              ▲              ▲                  │
│   ┌──────┴──────┐ ┌────┴────┐ ┌──────┴──────┐          │
│   │   SQLite    │ │ Ollama  │ │  Claude API │          │
│   │  Adapters   │ │ Adapter │ │   Adapter   │          │
│   │ (7 stores)  │ │(LLM+Emb)│ │   (LLM)    │          │
│   └─────────────┘ └─────────┘ └─────────────┘          │
│                                                          │
│   ┌──────────────────────────────────────────────────┐  │
│   │              Ingestion Pipeline                   │  │
│   │  Parse → Dedup → Normalize → Chunk → Embed → Store │
│   └──────────────────────────────────────────────────┘  │
│   ┌──────────────────────────────────────────────────┐  │
│   │              Analysis Pipeline                    │  │
│   │  Themes → Sentiment → Beliefs → Insights          │  │
│   └──────────────────────────────────────────────────┘  │
│   ┌──────────────────────────────────────────────────┐  │
│   │              RAG Query Pipeline                   │  │
│   │  Classify → Retrieve → RRF Fuse → Augment → Gen  │  │
│   └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### Port Interfaces

| Port | Purpose | Adapter |
|------|---------|---------|
| `IDocumentStore` | Raw document + chunk persistence | SQLite |
| `IVectorStore` | Embedding storage + similarity search | SQLite (cosine sim) |
| `IGraphStore` | Entity relationships (people, concepts) | SQLite (adjacency + CTE) |
| `IPageIndex` | Full-text search (BM25) | SQLite FTS5 |
| `ITimelineStore` | Temporal range queries | SQLite |
| `IMemoryStore` | Long-term facts (Mem0-style) | SQLite |
| `ILlmProvider` | Text generation + classification | Ollama / Claude |
| `IEmbeddingProvider` | Text → vector encoding | Ollama |
| `IAnalysisStage` | Pluggable analysis pipeline stage | Built-in stages |

---

## Project Structure

```
memory-palace/
├── src-tauri/                    # Rust backend
│   └── src/
│       ├── domain/               # Core domain models + port traits
│       │   ├── models/           # Document, Chunk, Entity, Theme, Memory, Insight
│       │   └── ports/            # 9 trait interfaces (hexagonal architecture)
│       ├── adapters/             # Port implementations
│       │   ├── sqlite/           # 7 SQLite adapters + migrations
│       │   └── llm/              # Ollama + Claude providers
│       ├── pipeline/             # Data processing
│       │   ├── ingestion/        # Source adapters, dedup, normalize, chunk, orchestrate
│       │   └── analysis/         # Theme, sentiment, belief, insight extraction
│       ├── query/                # RAG pipeline (retrieve → RRF fuse → generate)
│       ├── prompts/              # 7 versioned LLM prompt templates
│       └── commands/             # 15 Tauri IPC command handlers
│
├── src/                          # React frontend
│   ├── components/
│   │   ├── timeline/             # D3.js interactive timeline
│   │   ├── search/               # Multi-mode search interface
│   │   ├── ask/                  # RAG chat with citations
│   │   ├── insights/             # AI insight feed
│   │   ├── import/               # Import wizard
│   │   ├── memory/               # Memory facts browser
│   │   ├── settings/             # App settings
│   │   └── layout/               # AppShell + Sidebar
│   ├── stores/                   # Zustand state management
│   ├── lib/                      # Typed Tauri IPC command wrappers
│   └── types/                    # TypeScript domain type mirrors
│
├── .github/workflows/ci.yml     # Cross-platform CI (Ubuntu, Windows, macOS)
└── Memory_Palace_TRD_v1.docx.md # Technical Requirements Document
```

---

## Data Sources

### Supported (v0.1)
| Source | Format | What's Extracted |
|--------|--------|-----------------|
| **Obsidian** | Vault folder (.md) | Text, frontmatter dates, tags, wikilinks |
| **Markdown/Text** | Folder (.md, .txt) | Text, file modification dates |
| **Day One** | JSON export | Entries, timestamps, weather, location, tags |

### Planned (v0.2+)
| Source | Format |
|--------|--------|
| WhatsApp | ZIP (txt + media) |
| Telegram | JSON export |
| Twitter/X | ZIP archive |
| Reddit | GDPR CSV archive |
| Google Takeout | ZIP (mixed) |
| Notion | ZIP (Markdown/CSV) |

---

## Pipelines

### Ingestion
```
Source File/Folder
    │
    ▼
┌──────────────┐
│ Source Adapter │ ─── Obsidian / Markdown / Day One parser
└──────┬───────┘
       ▼
┌──────────────┐
│ Deduplication │ ─── SHA-256 content hash check
└──────┬───────┘
       ▼
┌──────────────┐
│ Normalizer    │ ─── Unicode NFC, timestamps to UTC, encoding cleanup
└──────┬───────┘
       ▼
┌──────────────┐
│ Chunker       │ ─── 512 tokens, 50 overlap, paragraph-aware
└──────┬───────┘
       ▼
┌──────────────┐
│ Store + Index │ ─── SQLite (docs + chunks + vectors + FTS5)
└──────────────┘
```

### Analysis
```
Imported Documents
    │
    ▼
┌──────────────────┐
│ Theme Extractor   │ ─── Monthly windows → LLM topic modeling → ThemeSnapshots
└──────┬───────────┘
       ▼
┌──────────────────┐
│ Sentiment Tracker │ ─── Per-chunk LLM classification → time series
└──────┬───────────┘
       ▼
┌──────────────────┐
│ Belief Extractor  │ ─── LLM extracts beliefs/preferences → MemoryFacts
└──────┬───────────┘
       ▼
┌──────────────────┐
│ Insight Generator │ ─── Synthesize top-N surprising observations
└──────────────────┘
```

### RAG Query
```
User Question
    │
    ▼
┌─────────────────┐
│ Query Classifier │ ─── Detect query type (semantic, temporal, entity, evolution)
└──────┬──────────┘
       ▼
┌─────────────────┐
│ Multi-Retrieval  │ ─── Parallel: FTS5 (BM25) + Vector (cosine)
└──────┬──────────┘
       ▼
┌─────────────────┐
│ Rank Fusion      │ ─── Reciprocal Rank Fusion (k=60)
└──────┬──────────┘
       ▼
┌─────────────────┐
│ Memory Augment   │ ─── Prepend relevant MemoryFacts as persistent context
└──────┬──────────┘
       ▼
┌─────────────────┐
│ LLM Generate     │ ─── Answer with source citations
└─────────────────┘
```

---

## Search Modes

| Mode | How It Works | Best For |
|------|-------------|----------|
| **Keyword** | SQLite FTS5 with BM25 ranking | Exact phrases, names, specific terms |
| **Semantic** | Embed query → cosine similarity on stored vectors | Conceptual searches ("times I felt lost") |
| **Hybrid** | Reciprocal Rank Fusion (BM25 + vector, k=60) | Best overall relevance |
| **RAG (Ask)** | Hybrid retrieval → memory augmentation → LLM generation | Conversational questions with cited answers |

---

## Testing

```bash
# Run all 49 Rust tests
cd src-tauri && cargo test --lib

# Type check frontend
npx tsc --noEmit

# Build frontend
npx vite build

# Lint Rust
cargo clippy --manifest-path src-tauri/Cargo.toml
```

### Test Coverage

| Module | Tests | What's Tested |
|--------|-------|---------------|
| SQLite DocumentStore | 4 | CRUD, hash lookup, chunks |
| SQLite MemoryStore | 3 | Store/recall, forget, contradict |
| SQLite GraphStore | 2 | Entity CRUD, neighbor traversal (recursive CTE) |
| SQLite TimelineStore | 2 | Monthly counts, date ranges |
| SQLite VectorStore | 5 | Cosine similarity, batch upsert, delete, blob roundtrip |
| SQLite Migrations | 2 | Fresh run, idempotent rerun |
| SQLite FTS5 Index | 1 | BM25 search via auto-sync triggers |
| Ingestion Chunker | 5 | Paragraph boundaries, overlap, force-split |
| Ingestion Normalizer | 4 | Unicode NFC, BOM, line endings, whitespace |
| Ingestion Dedup | 1 | Content hash deduplication |
| Ingestion Orchestrator | 2 | Full pipeline, dedup in pipeline |
| Obsidian Parser | 6 | Frontmatter, tags, wikilinks, dates |
| Day One Parser | 2 | Entry parsing, empty skip |
| Prompt Templates | 2 | Non-empty, variable substitution |
| Analysis Theme | 3 | Monthly windows, JSON parsing, extraction |
| Analysis Sentiment | 1 | Label parsing |
| Analysis Beliefs | 1 | JSON response parsing |
| Analysis Insights | 1 | JSON response parsing |
| RAG Pipeline | 1 | RRF fusion scoring logic |

---

## Tauri Commands (IPC)

| Command | Description |
|---------|-------------|
| `import_obsidian` | Import an Obsidian vault |
| `import_markdown` | Import a folder of .md/.txt files |
| `import_dayone` | Import a Day One JSON export |
| `keyword_search` | BM25-ranked full-text search |
| `semantic_search` | Vector cosine similarity search |
| `hybrid_search` | Reciprocal Rank Fusion search |
| `ask` | RAG pipeline — question answering with citations |
| `get_document_text` | Fetch raw text of a document |
| `get_timeline_data` | Monthly document counts + date range |
| `get_memory_facts` | List extracted memory facts (filterable by category) |
| `delete_memory_fact` | Delete a specific memory fact |
| `run_analysis` | Trigger full analysis pipeline |
| `test_ollama_connection` | Check if Ollama is running + list models |
| `get_app_stats` | Document count, memory facts count, date range |

---

## Configuration

Memory Palace stores its data in the platform-specific app data directory:

| Platform | Location |
|----------|----------|
| Windows | `%APPDATA%/com.memorypalace.app/` |
| macOS | `~/Library/Application Support/com.memorypalace.app/` |
| Linux | `~/.local/share/com.memorypalace.app/` |

### LLM Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| Ollama URL | `http://localhost:11434` | Local Ollama server address |
| Embedding model | `nomic-embed-text` | 1024-dim, 8192 token context |
| LLM model | `llama3.1:8b` | For theme/belief/insight extraction |
| Claude API key | (none) | Optional, stored in OS keychain |

---

## Roadmap

| Phase | Status | Key Deliverables |
|-------|--------|-----------------|
| **v0.1 — Foundation** | **Current** | Ingestion pipeline, SQLite storage, Ollama, timeline, search, RAG, analysis, 7 UI views |
| v0.2 — Expansion | Planned | WhatsApp/Telegram import, LanceDB vectors, entity explorer, graph visualization |
| v0.3 — Intelligence | Planned | Evolution detection, contradiction detection, narrative generation, advanced D3 timeline |
| v0.4 — Social | Planned | Twitter/Reddit/Instagram import, relationship mapping, mobile companion |
| v1.0 — Platform | Planned | Plugin system, export capabilities, encrypted sync, public release |

---

## Contributing

Memory Palace is open source. Contributions welcome!

The hexagonal architecture means adding a new source adapter, storage backend, or analysis stage only requires implementing one trait interface — no changes to business logic.

```bash
# Development setup
git clone https://github.com/laadtushar/MemPalace.git
cd MemPalace
npm install
cargo tauri dev
```

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## License

MIT License. See [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Built with Rust, React, and the belief that your personal data should work for <em>you</em> — privately.</sub>
</p>
