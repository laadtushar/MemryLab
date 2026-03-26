import Link from "next/link";

export default function ArchitecturePage() {
  return (
    <div>
      <div className="flex items-center gap-2 text-sm text-zinc-500 mb-8">
        <Link href="/docs" className="hover:text-white transition">
          Docs
        </Link>
        <span>/</span>
        <span className="text-white">Architecture</span>
      </div>

      <h1 className="text-4xl font-bold mb-6">Architecture</h1>
      <p className="text-zinc-400 text-lg mb-8">
        MemryLab follows a hexagonal (ports &amp; adapters) architecture with
        clear separation between domain logic, infrastructure, and UI.
      </p>

      <div className="space-y-12 text-zinc-300 leading-relaxed">
        {/* Tech Stack */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            Tech Stack
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {[
              ["Runtime", "Tauri 2.0 (Rust + WebView)"],
              ["Backend", "Rust with SQLite (rusqlite)"],
              ["Frontend", "React 18, TypeScript, Vite"],
              ["UI", "Tailwind CSS, Radix UI, Recharts"],
              ["Database", "SQLite with FTS5 full-text search"],
              ["Embeddings", "Local via LLM provider API"],
              ["Keychain", "OS-native (keytar via Tauri plugin)"],
              ["Build", "Tauri CLI, cross-platform bundles"],
            ].map(([label, value], i) => (
              <div
                key={i}
                className="flex items-start gap-3 glass rounded-lg p-4"
              >
                <span className="text-violet-400 font-medium min-w-[100px]">
                  {label}
                </span>
                <span className="text-zinc-300">{value}</span>
              </div>
            ))}
          </div>
        </section>

        {/* High-Level Architecture */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            High-Level Architecture
          </h2>
          <pre className="bg-zinc-900 rounded-lg p-6 text-sm overflow-x-auto font-mono">
            <code>{`┌─────────────────────────────────────────────┐
│                  Frontend                    │
│       React + TypeScript + Tailwind          │
│    Timeline │ Graph │ Chat │ Explorer         │
├─────────────────────────────────────────────┤
│              Tauri IPC Bridge                 │
│          (Commands / Events / State)          │
├─────────────────────────────────────────────┤
│                Rust Backend                   │
│                                               │
│  ┌─────────────┐  ┌──────────────────────┐   │
│  │   Domain     │  │     Pipeline         │   │
│  │   Models     │  │                      │   │
│  │  ─────────   │  │  Ingestion           │   │
│  │  Entry       │  │  ├── Source Adapters  │   │
│  │  Analysis    │  │  ├── ZIP Handler     │   │
│  │  Entity      │  │  └── Normalizer      │   │
│  │  Theme       │  │                      │   │
│  │  Insight     │  │  Analysis            │   │
│  │  Narrative   │  │  ├── Themes          │   │
│  └─────────────┘  │  ├── Sentiment        │   │
│                    │  ├── Beliefs          │   │
│  ┌─────────────┐  │  ├── Entities         │   │
│  │   Ports      │  │  ├── Insights        │   │
│  │  ─────────   │  │  ├── Contradictions  │   │
│  │  LLMPort     │  │  └── Narratives      │   │
│  │  StorePort   │  │                      │   │
│  │  EmbedPort   │  │  Search              │   │
│  │  GraphPort   │  │  ├── FTS5 keyword    │   │
│  └─────────────┘  │  ├── Vector semantic  │   │
│                    │  └── Hybrid fusion    │   │
│  ┌─────────────┐  └──────────────────────┘   │
│  │  Adapters    │                             │
│  │  ─────────   │                             │
│  │  SQLite DB   │                             │
│  │  LLM APIs    │                             │
│  │  OS Keychain │                             │
│  │  File System │                             │
│  └─────────────┘                              │
└─────────────────────────────────────────────┘`}</code>
          </pre>
        </section>

        {/* Data Flow */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">Data Flow</h2>
          <ol className="list-decimal list-inside space-y-3 text-zinc-400">
            <li>
              <strong className="text-white">Import:</strong> User selects a
              file/folder/ZIP. The ingestion pipeline identifies the source type
              using file signatures and structure patterns.
            </li>
            <li>
              <strong className="text-white">Parse:</strong> A source-specific
              adapter extracts entries (text + timestamp + metadata). ZIP files
              are handled transparently.
            </li>
            <li>
              <strong className="text-white">Normalize:</strong> All entries are
              normalized into a common{" "}
              <code className="px-1 py-0.5 rounded bg-zinc-900 text-xs">
                Entry
              </code>{" "}
              struct with source, timestamp, content, and metadata fields.
            </li>
            <li>
              <strong className="text-white">Store:</strong> Entries are inserted
              into SQLite with FTS5 indexing. Embeddings are generated via the
              configured LLM provider.
            </li>
            <li>
              <strong className="text-white">Analyze:</strong> The 8-stage
              analysis pipeline runs batch LLM calls, extracting themes,
              sentiment, entities, beliefs, insights, contradictions, and
              narratives.
            </li>
            <li>
              <strong className="text-white">Query:</strong> The frontend
              queries via Tauri IPC commands. Search combines FTS5 keyword
              matching with vector similarity for hybrid results.
            </li>
          </ol>
        </section>

        {/* Directory Structure */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            Directory Structure
          </h2>
          <pre className="bg-zinc-900 rounded-lg p-4 text-sm overflow-x-auto">
            <code>{`MemPalace/
├── src/                    # React frontend
│   ├── components/         # UI components
│   ├── pages/              # Route pages
│   ├── stores/             # Zustand state
│   └── lib/                # Utilities
├── src-tauri/
│   ├── src/
│   │   ├── domain/         # Core models & types
│   │   │   └── models/     # Entry, Analysis, Entity, etc.
│   │   ├── pipeline/       # Processing pipeline
│   │   │   ├── ingestion/  # Source adapters, ZIP handler
│   │   │   ├── analysis/   # LLM analysis stages
│   │   │   └── search/     # FTS5, vector, hybrid
│   │   ├── ports/          # Abstract interfaces
│   │   ├── adapters/       # SQLite, LLM, Keychain impls
│   │   └── commands/       # Tauri IPC command handlers
│   ├── Cargo.toml
│   └── tauri.conf.json
└── package.json`}</code>
          </pre>
        </section>

        {/* Design Principles */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            Design Principles
          </h2>
          <ul className="list-disc list-inside space-y-3 text-zinc-400">
            <li>
              <strong className="text-white">Privacy by default:</strong> All
              processing is local. The only network calls are to LLM APIs, and
              only the minimum context needed is sent.
            </li>
            <li>
              <strong className="text-white">Hexagonal architecture:</strong>{" "}
              Domain models have no dependencies on infrastructure. Ports define
              abstract interfaces; adapters implement them.
            </li>
            <li>
              <strong className="text-white">Adapter pattern for sources:</strong>{" "}
              Each import source is a self-contained adapter implementing a
              common trait. Adding a new source requires no changes to existing
              code.
            </li>
            <li>
              <strong className="text-white">Provider-agnostic LLM:</strong> The
              LLM port abstracts away provider specifics. Switching from Gemini
              to Groq is a single setting change.
            </li>
            <li>
              <strong className="text-white">Tiny binary:</strong> Tauri 2.0
              compiles to native code with the system WebView. No bundled
              Chromium, no V8.
            </li>
          </ul>
        </section>
      </div>

      <div className="flex items-center justify-between mt-16 pt-8 border-t border-zinc-800">
        <Link
          href="/docs/ai-providers"
          className="text-sm text-zinc-500 hover:text-white transition"
        >
          &larr; AI Providers
        </Link>
        <Link
          href="/docs/contributing"
          className="text-sm text-violet-400 hover:text-violet-300 transition"
        >
          Contributing &rarr;
        </Link>
      </div>
    </div>
  );
}
