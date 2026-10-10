import Link from "next/link";
import { pageMetadata } from "@/lib/seo";
import { WINDOWS_INSTALLER_SIZE } from "@/lib/site";

export const metadata = pageMetadata({
  title: "MemryLab Architecture: Hexagonal Rust and React App",
  description:
    "How MemryLab is built: Tauri 2, a Rust backend with hexagonal ports and adapters, SQLite with FTS5, an 8-stage analysis pipeline and RAG search.",
  path: "/docs/architecture",
});

function Arrow() {
  return <span className="text-zinc-600 text-lg">→</span>;
}

function PipelineStep({ label, color = "bg-violet-600/20 border-violet-500/30 text-violet-300" }: { label: string; color?: string }) {
  return <span className={`px-2.5 py-1 rounded-md text-xs font-medium border ${color}`}>{label}</span>;
}

export default function ArchitecturePage() {
  return (
    <div>
      <div className="flex items-center gap-2 text-sm text-zinc-500 mb-8">
        <Link href="/docs" className="hover:text-white transition">Docs</Link>
        <span>/</span>
        <span className="text-white">Architecture</span>
      </div>

      <h1 className="text-4xl font-bold mb-6">Architecture</h1>
      <p className="text-zinc-400 text-lg mb-8">
        MemryLab follows a strict <strong className="text-white">hexagonal (ports &amp; adapters)</strong> architecture.
        The core domain has zero knowledge of specific databases, LLM providers, or UI frameworks.
      </p>

      <div className="space-y-12 text-zinc-300 leading-relaxed">

        {/* System Architecture Diagram */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-6">System Overview</h2>
          <div className="rounded-xl border border-zinc-800 bg-zinc-950 p-6 space-y-6">

            {/* Frontend Layer */}
            <div className="rounded-lg border border-violet-500/30 bg-violet-600/10 p-4">
              <div className="text-xs font-semibold text-violet-400 uppercase tracking-wider mb-3">React 19 Frontend</div>
              <div className="flex flex-wrap gap-1.5">
                {["Timeline","Activity","Search","Ask/Chat","Insights","Evolution","Import","Memory","Entities","Graph","Logs","Settings"].map(v=>(
                  <span key={v} className="px-2 py-0.5 text-[10px] rounded bg-violet-600/20 text-violet-300 border border-violet-500/20">{v}</span>
                ))}
              </div>
            </div>

            {/* IPC Layer */}
            <div className="flex items-center justify-center gap-4 text-xs text-zinc-500">
              <div className="flex-1 h-px bg-zinc-800"/>
              <span className="px-3 py-1 rounded-full bg-zinc-900 border border-zinc-800">Tauri IPC — 60+ Commands + Events</span>
              <div className="flex-1 h-px bg-zinc-800"/>
            </div>

            {/* Domain Layer */}
            <div className="rounded-lg border border-blue-500/30 bg-blue-600/10 p-4">
              <div className="text-xs font-semibold text-blue-400 uppercase tracking-wider mb-3">Domain (Pure Logic)</div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <div className="text-[10px] text-zinc-500 uppercase tracking-wider mb-1.5">Models</div>
                  <div className="flex flex-wrap gap-1">
                    {["Document","Chunk","Entity","Relationship","Theme","Memory","Insight","Narrative","Contradiction","Sentiment"].map(m=>(
                      <span key={m} className="px-1.5 py-0.5 text-[9px] rounded bg-blue-600/20 text-blue-300 border border-blue-500/20">{m}</span>
                    ))}
                  </div>
                </div>
                <div>
                  <div className="text-[10px] text-zinc-500 uppercase tracking-wider mb-1.5">Ports (Interfaces)</div>
                  <div className="flex flex-wrap gap-1">
                    {["IDocumentStore","IVectorStore","IGraphStore","ILlmProvider","IEmbeddingProvider","IMemoryStore","IPageIndex","ITimelineStore"].map(p=>(
                      <span key={p} className="px-1.5 py-0.5 text-[9px] rounded bg-blue-600/20 text-blue-300 border border-blue-500/20 font-mono">{p}</span>
                    ))}
                  </div>
                </div>
              </div>
            </div>

            {/* Adapters Layer */}
            <div className="grid grid-cols-4 gap-3">
              {[
                {title:"SQLite Adapters", desc:"7 stores + FTS5", color:"border-zinc-700 bg-zinc-900"},
                {title:"Ollama", desc:"Local LLM + Embeddings", color:"border-emerald-500/30 bg-emerald-600/10"},
                {title:"OpenAI-Compat", desc:"7 cloud presets + custom endpoint", color:"border-cyan-500/30 bg-cyan-600/10"},
                {title:"Claude API", desc:"Anthropic adapter", color:"border-amber-500/30 bg-amber-600/10"},
              ].map(a=>(
                <div key={a.title} className={`rounded-lg border p-3 ${a.color}`}>
                  <div className="text-xs font-semibold text-zinc-200">{a.title}</div>
                  <div className="text-[10px] text-zinc-500 mt-0.5">{a.desc}</div>
                </div>
              ))}
            </div>

            {/* Pipeline Layer */}
            <div className="rounded-lg border border-emerald-500/30 bg-emerald-600/10 p-4 space-y-4">
              <div className="text-xs font-semibold text-emerald-400 uppercase tracking-wider">Pipelines</div>

              <div>
                <div className="text-[10px] text-zinc-500 mb-2">Ingestion Pipeline</div>
                <div className="flex flex-wrap items-center gap-1.5">
                  {["Detect","Parse","Dedup","Normalize","Chunk","Embed","Store","Index"].map((s,i,arr)=>(
                    <span key={s} className="contents">
                      <PipelineStep label={s} color="bg-emerald-600/20 border-emerald-500/30 text-emerald-300"/>
                      {i<arr.length-1 && <Arrow/>}
                    </span>
                  ))}
                </div>
              </div>

              <div>
                <div className="text-[10px] text-zinc-500 mb-2">Analysis Pipeline (8 stages)</div>
                <div className="flex flex-wrap items-center gap-1.5">
                  {["Themes","Sentiment","Beliefs","Entities","Insights","Contradictions","Evolution","Narratives"].map((s,i,arr)=>(
                    <span key={s} className="contents">
                      <PipelineStep label={s}/>
                      {i<arr.length-1 && <Arrow/>}
                    </span>
                  ))}
                </div>
              </div>

              <div>
                <div className="text-[10px] text-zinc-500 mb-2">RAG Query Pipeline</div>
                <div className="flex flex-wrap items-center gap-1.5">
                  {["Classify","Retrieve","RRF Fuse","Memory Augment","LLM Generate","Citations"].map((s,i,arr)=>(
                    <span key={s} className="contents">
                      <PipelineStep label={s} color="bg-cyan-600/20 border-cyan-500/30 text-cyan-300"/>
                      {i<arr.length-1 && <Arrow/>}
                    </span>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Tech Stack */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">Tech Stack</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {[
              ["App Shell", "Tauri 2.0 (Rust + the system WebView)"],
              ["Backend", "Rust with async-trait, tokio"],
              ["Frontend", "React 19, TypeScript, Vite 8"],
              ["Styling", "Tailwind CSS 4, Lucide Icons"],
              ["Visualization", "D3.js v7 (timeline, graph)"],
              ["Database", "SQLite (WAL mode) + FTS5"],
              ["Vector Store", "SQLite (cosine similarity)"],
              ["Logging", "tracing + tracing-appender (daily rotation)"],
              ["Security", "OS Keychain (keyring crate)"],
              ["Build", "Tauri CLI, NSIS/MSI/DMG/AppImage/deb/RPM"],
            ].map(([label, value], i) => (
              <div key={i} className="flex items-start gap-3 rounded-lg border border-zinc-800 bg-zinc-900/50 p-3">
                <span className="text-violet-400 font-medium text-sm min-w-[110px]">{label}</span>
                <span className="text-zinc-400 text-sm">{value}</span>
              </div>
            ))}
          </div>
        </section>

        {/* Data Flow */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">Data Flow</h2>
          <ol className="list-decimal list-inside space-y-3 text-zinc-400">
            <li><strong className="text-white">Import:</strong> User drops a file/folder/ZIP. The system auto-detects the source using confidence scoring across 30+ adapters. A generic sweep catches remaining files.</li>
            <li><strong className="text-white">Parse:</strong> Platform-specific adapter extracts text, timestamps, participants, and metadata. ZIP files are extracted transparently.</li>
            <li><strong className="text-white">Process:</strong> Documents are deduplicated (SHA-256), normalized (Unicode NFC), chunked (512 tokens, paragraph-aware), and embedded via the configured AI provider.</li>
            <li><strong className="text-white">Analyze:</strong> The 8-stage analysis pipeline extracts themes, sentiment, beliefs, entities, insights, contradictions, and generates narratives — all via LLM.</li>
            <li><strong className="text-white">Query:</strong> Hybrid search combines FTS5 keyword + vector similarity via Reciprocal Rank Fusion. RAG augments with memory facts before LLM generation.</li>
            <li><strong className="text-white">Log:</strong> Every action (import, analysis, search, config change) is logged to the activity history with full results and timing.</li>
          </ol>
        </section>

        {/* Directory Structure */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">Directory Structure</h2>
          <pre className="bg-zinc-900 rounded-lg p-4 text-sm overflow-x-auto">
            <code>{`MemryLab/
├── src/                          # React frontend
│   ├── components/
│   │   ├── timeline/             # D3 zoomable timeline
│   │   ├── graph/                # D3 force-directed graph
│   │   ├── ask/                  # RAG chat with history
│   │   ├── import/               # 30+ source import wizard
│   │   ├── activity/             # Activity history feed
│   │   ├── logs/                 # Application log viewer
│   │   ├── onboarding/           # First-run wizard
│   │   └── settings/             # Provider config + about
│   ├── stores/                   # Zustand state management
│   └── lib/tauri.ts              # Type-safe IPC bindings (60+ commands)
├── src-tauri/src/
│   ├── domain/
│   │   ├── models/               # Document, Entity, Theme, Memory, etc.
│   │   └── ports/                # 9 trait interfaces
│   ├── adapters/
│   │   ├── sqlite/               # 7 SQLite stores + migrations
│   │   ├── llm/                  # Ollama, Claude, OpenAI-compat, usage logger
│   │   └── keychain.rs           # OS credential store
│   ├── pipeline/
│   │   ├── ingestion/            # 30+ source adapters + orchestrator
│   │   ├── analysis/             # 8 analysis stages
│   │   └── pii_detector.rs       # PII regex scanner
│   ├── query/                    # RAG pipeline with RRF fusion
│   ├── prompts/                  # Versioned prompt templates
│   └── commands/                 # 60+ Tauri command handlers
├── website/                      # Next.js marketing site
└── paper/                        # Research paper (LaTeX + PDF)`}</code>
          </pre>
        </section>

        {/* Design Principles */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">Design Principles</h2>
          <ul className="list-disc list-inside space-y-3 text-zinc-400">
            <li><strong className="text-white">Privacy by default:</strong> All processing is local. Network calls only go to the user&apos;s chosen LLM provider with minimum context.</li>
            <li><strong className="text-white">Hexagonal architecture:</strong> Domain models have zero dependencies on infrastructure. Ports define interfaces; adapters implement them.</li>
            <li><strong className="text-white">Two-pass exploratory import:</strong> Platform adapter runs first, then a generic sweep catches all remaining text files. No file left behind.</li>
            <li><strong className="text-white">Provider-agnostic LLM:</strong> One trait interface, 9 providers. Switching from Gemini to Groq is a single click.</li>
            <li><strong className="text-white">Tiny binary:</strong> {WINDOWS_INSTALLER_SIZE} Windows installer. Tauri 2.0 uses the system WebView — no bundled Chromium.</li>
            <li><strong className="text-white">Graceful degradation:</strong> If embeddings fail, search still works via FTS5. If LLM is offline, import still succeeds.</li>
          </ul>
        </section>
      </div>

      <div className="flex items-center justify-between mt-16 pt-8 border-t border-zinc-800">
        <Link href="/docs/ai-providers" className="text-sm text-zinc-500 hover:text-white transition">&larr; AI Providers</Link>
        <Link href="/docs/contributing" className="text-sm text-violet-400 hover:text-violet-300 transition">Contributing &rarr;</Link>
      </div>
    </div>
  );
}
