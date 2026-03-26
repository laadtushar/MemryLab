<p align="center">
  <img src="https://img.shields.io/badge/Memory_Palace-0.1.0-8b5cf6?style=for-the-badge" alt="Memory Palace" />
</p>

<h3 align="center">A searchable, visual timeline of how your thinking evolved.</h3>

<!-- Badges: Project -->
<p align="center">
  <a href="https://github.com/laadtushar/MemPalace/actions/workflows/ci.yml"><img src="https://github.com/laadtushar/MemPalace/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <img src="https://img.shields.io/badge/status-MVP-yellow?style=flat-square" alt="Status" />
  <img src="https://img.shields.io/badge/license-MIT-green?style=flat-square" alt="License" />
  <img src="https://img.shields.io/badge/PRs-welcome-brightgreen?style=flat-square" alt="PRs Welcome" />
  <img src="https://img.shields.io/badge/installer-4.3MB-blue?style=flat-square" alt="Installer Size" />
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
  <img src="https://img.shields.io/badge/LLM_Providers-9_(8_free)-blueviolet?style=flat-square" alt="LLM Providers" />
  <img src="https://img.shields.io/badge/Import_Sources-30+-orange?style=flat-square" alt="Import Sources" />
  <img src="https://img.shields.io/badge/Privacy-Local_First-blueviolet?style=flat-square" alt="Privacy" />
  <img src="https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=flat-square" alt="Platform" />
  <img src="https://img.shields.io/badge/Architecture-Hexagonal-informational?style=flat-square" alt="Architecture" />
  <img src="https://img.shields.io/badge/Rust_Files-120+-CE422B?style=flat-square" alt="Rust Files" />
</p>

---

## What is Memory Palace?

**Memory Palace** is a privacy-first native desktop application that ingests your entire digital footprint -- journal entries, chat exports, social media archives, notes, emails -- and constructs a searchable, visual timeline of how your thinking, interests, and emotional patterns have evolved over time. It supports 30+ import sources with automatic format detection, runs an 8-stage AI analysis pipeline, and provides keyword, semantic, and hybrid search alongside a RAG-powered chat interface. Everything runs locally in a single SQLite database with zero telemetry, zero cloud dependency, and a 4.3 MB installer.

---

## Key Features

### Import
- **30+ source adapters** with auto-detection -- drop a ZIP or folder and Memory Palace identifies the format
- Obsidian vaults, Markdown/text files, Day One journals, and 27 platform data exports
- ZIP archive extraction with nested format handling
- SHA-256 content deduplication -- reimporting the same data is safe
- Progress tracking with real-time event streaming

### Analysis (8-Stage Pipeline)
| Stage | What It Does |
|-------|-------------|
| **Theme Extraction** | Monthly time-window topic modeling via LLM |
| **Sentiment Tracking** | Per-chunk emotional classification across time |
| **Belief Extraction** | Surfaces beliefs, preferences, and self-descriptions |
| **Entity Extraction** | Identifies people, places, concepts, and organizations |
| **Insight Generation** | Synthesizes surprising observations about personal evolution |
| **Contradiction Detection** | Finds conflicts between current and past beliefs |
| **Evolution Diffing** | Tracks how topics and opinions shift over time |
| **Narrative Generation** | Produces written summaries of personal growth arcs |

### Query
- **Keyword Search** -- BM25-ranked full-text search via SQLite FTS5
- **Semantic Search** -- Vector cosine similarity using local or cloud embeddings
- **Hybrid Search** -- Reciprocal Rank Fusion (k=60) combining both
- **RAG Chat** -- Natural language questions with source citations, memory-augmented context

### Visualizations
- **Zoomable Timeline** -- D3.js interactive bar chart with monthly document density, zoom/pan, activity tables
- **Force-Directed Graph** -- Entity relationship explorer with node detail panels
- **Evolution Explorer** -- Compare beliefs across time periods, view narrative arcs
- **Entity Explorer** -- Browse and query extracted people, places, concepts

### Privacy and Security
- **100% local by default** -- all data stored and processed on-device
- **Zero telemetry** -- no analytics, no phone-home, no tracking
- **OS keychain integration** -- API keys stored in Windows Credential Manager / macOS Keychain / Linux Secret Service
- **SQLCipher-ready** -- encrypted database support (bundled feature flag)
- **Passphrase protection** -- optional database encryption dialog
- **No external services required** -- fully functional with Ollama alone

### AI Providers
- **9 providers** with one-click preset setup, **8 with free tiers**
- **Universal OpenAI-compatible adapter** -- works with any endpoint that speaks the OpenAI API format
- Runtime provider switching -- change models without restarting
- Usage logging for token tracking

---

<!-- Screenshots coming soon -->

---

## Quick Start

### Option A: Download Installer

Download the latest release from the [Releases](https://github.com/laadtushar/MemPalace/releases) page.

| Platform | Installer | Size |
|----------|-----------|------|
| Windows | `.exe` (NSIS) | ~4.3 MB |
| macOS | `.dmg` | Coming soon |
| Linux | `.AppImage` / `.deb` | Coming soon |

### Option B: Build from Source

**Prerequisites:** [Rust](https://rustup.rs/) 1.70+, [Node.js](https://nodejs.org/) 18+, [Ollama](https://ollama.com/) (optional, for local LLM)

```bash
git clone https://github.com/laadtushar/MemPalace.git
cd MemPalace
npm install

# (Optional) Pull local models
ollama pull nomic-embed-text
ollama pull llama3.1:8b

# Development
cargo tauri dev

# Production build
cargo tauri build
```

### First Run

1. Open the app and go to **Settings** to pick an AI provider (Ollama, Groq, Gemini, etc.)
2. Click **Import** in the sidebar -- select a source type or drop a ZIP/folder
3. Run **Analysis** from the Insights view to extract themes, beliefs, and entities
4. **Search**, **Ask**, or explore the **Timeline**, **Graph**, and **Evolution** views

---

## Supported Import Sources

### Notes and Journals

| Source | Format | What's Extracted |
|--------|--------|-----------------|
| **Obsidian** | Vault folder (`.md`) | Text, frontmatter dates, tags, wikilinks |
| **Markdown / Text** | Folder (`.md`, `.txt`) | Text, file modification dates |
| **Day One** | JSON export | Entries, timestamps, weather, location, tags |
| **Notion** | ZIP (Markdown/CSV) | Pages, databases, timestamps |
| **Evernote** | `.enex` XML export | Notes, tags, creation dates |
| **Apple Notes** | Export folder | Note text, dates |

### Social Media

| Source | Format | Takeout Link |
|--------|--------|-------------|
| **Facebook** | [JSON ZIP](https://www.facebook.com/dyi/?referrer=yfi_settings) | Posts, messages, comments |
| **Instagram** | [JSON ZIP](https://www.instagram.com/download/request/) | Posts, messages, stories |
| **Twitter / X** | [ZIP archive](https://twitter.com/settings/download_your_data) | Tweets, DMs, likes |
| **Reddit** | [GDPR CSV](https://www.reddit.com/settings/data-request) | Posts, comments, saved |
| **TikTok** | [JSON export](https://www.tiktok.com/setting/download-your-data) | Videos, comments, messages |
| **Snapchat** | [JSON export](https://accounts.snapchat.com/accounts/downloadmydata) | Messages, memories, stories |
| **LinkedIn** | [CSV export](https://www.linkedin.com/mypreferences/d/download-my-data) | Posts, messages, connections |
| **Pinterest** | ZIP export | Pins, boards |
| **Tumblr** | ZIP export | Posts, messages |
| **Threads** | JSON export | Posts, replies |
| **Mastodon** | JSON export | Toots, follows |
| **Bluesky** | JSON export | Posts, follows |
| **Substack** | Export files | Posts, newsletters |
| **Medium** | Export ZIP | Articles, responses |

### Messaging

| Source | Format | Takeout Link |
|--------|--------|-------------|
| **WhatsApp** | [TXT/ZIP export](https://faq.whatsapp.com/1180414079177245/) | Messages, timestamps, participants |
| **Telegram** | [JSON export](https://telegram.org/blog/export-and-more) | Messages, channels, groups |
| **Discord** | [Data package](https://support.discord.com/hc/en-us/articles/360004027692) | Messages, servers, DMs |
| **Slack** | [ZIP export](https://slack.com/help/articles/201658943) | Messages, channels, threads |
| **Signal** | Backup export | Messages, conversations |

### Media and Streaming

| Source | Format | What's Extracted |
|--------|--------|-----------------|
| **YouTube** | [Google Takeout](https://takeout.google.com/) | Watch history, comments, playlists |
| **Spotify** | [Data export](https://www.spotify.com/account/privacy/) | Listening history, playlists |
| **Netflix** | [Account data](https://www.netflix.com/account/getmyinfo) | Viewing history, ratings |

### Productivity and Cloud

| Source | Format | What's Extracted |
|--------|--------|-----------------|
| **Google Takeout** | [ZIP (mixed)](https://takeout.google.com/) | Gmail, Drive, Keep, Calendar, etc. |
| **Microsoft** | [Data export](https://account.microsoft.com/privacy) | Outlook, OneDrive, Teams |
| **Amazon** | [Data request](https://www.amazon.com/gp/privacycentral/dsar/preview.html) | Orders, searches, Alexa |
| **Generic** | Any text files | Auto-detected fallback adapter |

---

## AI Provider Setup

All providers are preconfigured with one-click presets. Select one in **Settings**.

| Provider | Free Tier | Embeddings | Default Model | Signup |
|----------|-----------|------------|---------------|--------|
| **Ollama** (local) | Unlimited | nomic-embed-text | Llama 3.1 8B | [ollama.com](https://ollama.com/download) |
| **OpenRouter** | 29 free models | -- | Llama 3.3 70B | [openrouter.ai](https://openrouter.ai/) |
| **Groq** | 30 RPM | -- | Llama 3.3 70B | [console.groq.com](https://console.groq.com/) |
| **Google Gemini** | 10 RPM | text-embedding-004 | Gemini 2.5 Flash | [aistudio.google.com](https://aistudio.google.com/) |
| **Cerebras** | 1M tokens/day | -- | Llama 3.3 70B | [cloud.cerebras.ai](https://cloud.cerebras.ai/) |
| **Mistral** | 1B tokens/month | mistral-embed | Mistral Small | [console.mistral.ai](https://console.mistral.ai/) |
| **SambaNova** | Indefinite | -- | Llama 3.3 70B | [cloud.sambanova.ai](https://cloud.sambanova.ai/) |
| **Cohere** | 1K calls/month | embed-v4.0 | Command A | [dashboard.cohere.com](https://dashboard.cohere.com/) |
| **Claude** (Anthropic) | Pay-per-use | -- | Claude Sonnet 4 | [console.anthropic.com](https://console.anthropic.com/) |

Any OpenAI-compatible endpoint can also be configured manually with a custom base URL.

---

## Architecture

Memory Palace follows a strict **hexagonal (ports and adapters) architecture**. The core domain has zero knowledge of specific databases, LLM providers, or UI frameworks.

```
+---------------------------------------------------------------+
|                       React 19 Frontend                       |
|  Timeline | Search | Ask | Insights | Evolution | Import      |
|  Memory   | Entities | Graph | Settings                      |
+-------------------------------+-------------------------------+
|     Tauri IPC Commands (30+)  |    Events (import progress)   |
+-------------------------------+-------------------------------+
|                                                               |
|   +-------------------------------------------------------+   |
|   |                  Domain (Pure Logic)                   |   |
|   |  Models: Document, Chunk, Entity, Theme, Memory,      |   |
|   |          Insight, Narrative, Contradiction, Sentiment  |   |
|   |  Ports:  IDocumentStore, IVectorStore, IGraphStore,    |   |
|   |          ILlmProvider, IEmbeddingProvider, IMemoryStore|   |
|   |          IPageIndex, ITimelineStore, IAnalysisStage    |   |
|   +-------------------------------------------------------+   |
|          ^               ^               ^                    |
|   +------+------+  +----+----+  +-------+--------+           |
|   |   SQLite    |  | Ollama  |  | OpenAI-Compat  |           |
|   |  Adapters   |  | Adapter |  |    Adapter     |           |
|   |  (7 stores) |  |(LLM+Emb)|  | (8 providers)  |           |
|   +-------------+  +---------+  +-------+--------+           |
|                                         |                     |
|                                  +------+------+              |
|                                  | Claude API  |              |
|                                  |   Adapter   |              |
|                                  +-------------+              |
|                                                               |
|   +-------------------------------------------------------+   |
|   |               Ingestion Pipeline                      |   |
|   |  Detect -> Parse -> Dedup -> Normalize -> Chunk ->    |   |
|   |  Embed -> Store -> Index                              |   |
|   +-------------------------------------------------------+   |
|   +-------------------------------------------------------+   |
|   |               Analysis Pipeline (8 stages)            |   |
|   |  Themes -> Sentiment -> Beliefs -> Entities ->        |   |
|   |  Insights -> Contradictions -> Evolution -> Narrative  |   |
|   +-------------------------------------------------------+   |
|   +-------------------------------------------------------+   |
|   |               RAG Query Pipeline                      |   |
|   |  Classify -> Retrieve -> RRF Fuse -> Memory Augment   |   |
|   |  -> LLM Generate (with citations)                     |   |
|   +-------------------------------------------------------+   |
+---------------------------------------------------------------+
```

### Port Interfaces (9)

| Port | Purpose | Adapter |
|------|---------|---------|
| `IDocumentStore` | Document + chunk persistence | SQLite |
| `IVectorStore` | Embedding storage + similarity search | SQLite (cosine sim) |
| `IGraphStore` | Entity relationships | SQLite (adjacency + recursive CTE) |
| `IPageIndex` | Full-text search (BM25) | SQLite FTS5 |
| `ITimelineStore` | Temporal range queries | SQLite |
| `IMemoryStore` | Long-term facts (Mem0-style) | SQLite |
| `ILlmProvider` | Text generation + classification | Ollama / Claude / OpenAI-compat |
| `IEmbeddingProvider` | Text to vector encoding | Ollama / Gemini / Mistral / Cohere |
| `IAnalysisStage` | Pluggable analysis pipeline stage | 8 built-in stages |

---

## Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **App Shell** | [Tauri 2.0](https://v2.tauri.app/) | Native desktop (Rust backend + web frontend) |
| **Backend** | Rust (2021 edition) | Memory safety, performance, async runtime |
| **Frontend** | React 19 + TypeScript 6 | Component-based UI with type safety |
| **Styling** | Tailwind CSS 4 + Lucide Icons | Dark/light theme, utility-first |
| **State** | Zustand 5 | Lightweight TypeScript-native state management |
| **Visualization** | D3.js 7 | Timeline, graph, and evolution visualizations |
| **Database** | SQLite (rusqlite, bundled) | Single-file storage for everything |
| **Full-Text Search** | SQLite FTS5 | BM25-ranked keyword search with auto-sync triggers |
| **Vector Search** | SQLite + cosine similarity | Semantic search with local embeddings |
| **Graph Storage** | SQLite adjacency model | Entity relationships with recursive CTE traversal |
| **Local LLM** | [Ollama](https://ollama.com/) | Embeddings + completions, fully offline |
| **Cloud LLM** | OpenAI-compatible API | 8 cloud providers via universal adapter |
| **Secrets** | OS Keychain (keyring) | Windows Credential Manager / macOS Keychain / Linux Secret Service |
| **Build** | Vite 8 + Cargo | Fast dev builds, cross-platform packaging |
| **CI/CD** | GitHub Actions | Cross-platform build + test (Ubuntu, Windows, macOS) |

---

## Views

Memory Palace has **10 views**, each accessible from the sidebar:

| View | Description |
|------|-------------|
| **Timeline** | D3.js interactive bar chart with zoom/pan, monthly document counts, activity table, and time boundary markers |
| **Search** | Multi-mode search (keyword / semantic / hybrid) with result cards and full document viewer |
| **Ask** | Chat-style RAG interface with suggested questions, source citations, and memory-augmented context |
| **Insights** | AI-generated observations about personal evolution with one-click analysis trigger |
| **Evolution** | Compare beliefs and themes across time periods, view narrative arcs, diff mode |
| **Import** | Guided wizard with 30+ source adapters, auto-detection, ZIP handling, and progress tracking |
| **Memory** | Filterable browser of extracted facts, beliefs, preferences, and self-descriptions with delete controls |
| **Entities** | Browse and query extracted people, places, concepts, and organizations |
| **Graph** | Force-directed entity relationship explorer with node detail panel |
| **Settings** | Provider selection (9 presets), model configuration, connection testing, export, and app statistics |

---

## Development

### Prerequisites

- [Rust](https://rustup.rs/) 1.70+
- [Node.js](https://nodejs.org/) 18+
- [Ollama](https://ollama.com/) (optional, for local LLM)

### Setup

```bash
git clone https://github.com/laadtushar/MemPalace.git
cd MemPalace
npm install
```

### Dev Mode

```bash
cargo tauri dev
```

### Build for Production

```bash
cargo tauri build
```

### Run Tests

```bash
# Rust tests
cd src-tauri && cargo test --lib

# TypeScript type check
npx tsc --noEmit

# Lint
cargo clippy --manifest-path src-tauri/Cargo.toml
```

---

## Project Structure

```
MemPalace/
├── src-tauri/                       # Rust backend (120+ source files)
│   └── src/
│       ├── domain/                  # Core domain models + port traits
│       │   ├── models/              # Document, Chunk, Entity, Theme, Memory, Insight,
│       │   │                        # Narrative, Contradiction, Sentiment, TimeBoundary
│       │   └── ports/               # 9 trait interfaces (hexagonal architecture)
│       ├── adapters/                # Port implementations
│       │   ├── sqlite/              # 7 SQLite adapters + migrations
│       │   ├── llm/                 # Ollama, Claude, OpenAI-compat providers + usage logger
│       │   └── keychain/            # OS keychain adapter for secret storage
│       ├── pipeline/
│       │   ├── ingestion/           # Source adapters (30+), dedup, normalize, chunk, ZIP handler
│       │   └── analysis/            # 8 stages: themes, sentiment, beliefs, entities,
│       │                            # insights, contradictions, evolution, narratives
│       ├── query/                   # RAG pipeline (retrieve -> RRF fuse -> augment -> generate)
│       ├── prompts/                 # Versioned LLM prompt templates
│       └── commands/                # Tauri IPC command handlers
│
├── src/                             # React 19 frontend
│   ├── components/
│   │   ├── timeline/                # D3.js interactive timeline
│   │   ├── search/                  # Multi-mode search interface
│   │   ├── ask/                     # RAG chat with citations
│   │   ├── insights/                # AI insight feed
│   │   ├── evolution/               # Evolution explorer with diff + narrative views
│   │   ├── import/                  # Import wizard with auto-detect
│   │   ├── memory/                  # Memory facts browser
│   │   ├── entities/                # Entity explorer
│   │   ├── graph/                   # Force-directed graph + node detail panel
│   │   ├── settings/                # Provider config, export, stats
│   │   ├── auth/                    # Passphrase dialog
│   │   ├── layout/                  # AppShell + Sidebar
│   │   ├── shared/                  # Error boundary, common components
│   │   └── ui/                      # Design system primitives
│   ├── stores/                      # Zustand state management
│   ├── lib/                         # Typed Tauri IPC command wrappers
│   └── types/                       # TypeScript domain type mirrors
│
├── .github/workflows/ci.yml        # Cross-platform CI (Ubuntu, Windows, macOS)
└── package.json
```

---

## Contributing

Memory Palace is open source. Contributions welcome!

The hexagonal architecture means adding a new source adapter, storage backend, or analysis stage only requires implementing one trait interface -- no changes to business logic.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

---

## Roadmap

### Done (v0.1)

- [x] 30+ source adapters with auto-detection and ZIP handling
- [x] 8-stage analysis pipeline (themes, sentiment, beliefs, entities, insights, contradictions, evolution, narratives)
- [x] Keyword, semantic, and hybrid search
- [x] RAG chat with source citations
- [x] D3.js zoomable timeline with time boundaries
- [x] Force-directed entity graph explorer
- [x] Evolution explorer with diff and narrative views
- [x] 9 LLM providers (8 free) with one-click setup
- [x] OS keychain integration for API key storage
- [x] Memory export (JSON, Markdown)
- [x] Dark/light theme toggle
- [x] Keyboard shortcuts
- [x] Cross-platform CI (Windows, macOS, Linux)
- [x] 10 UI views

### Next

- [ ] Plugin system for community adapters
- [ ] LanceDB vector backend (upgradeable from SQLite)
- [ ] Encrypted sync across devices
- [ ] Mobile companion app
- [ ] Advanced D3 timeline (events, annotations, overlays)
- [ ] Relationship mapping and social graph
- [ ] Batch re-analysis with progress
- [ ] PDF and image OCR import

---

## Configuration

Memory Palace stores data in the platform-specific app data directory:

| Platform | Location |
|----------|----------|
| Windows | `%APPDATA%/com.memorypalace.app/` |
| macOS | `~/Library/Application Support/com.memorypalace.app/` |
| Linux | `~/.local/share/com.memorypalace.app/` |

---

## License

MIT License. See [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Built with Rust, React, and the belief that your personal data should work for <em>you</em> -- privately.</sub>
</p>
