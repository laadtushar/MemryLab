import Link from "next/link";

export default function GettingStartedPage() {
  return (
    <div>
      {/* Breadcrumbs */}
      <div className="flex items-center gap-2 text-sm text-zinc-500 mb-8">
        <Link href="/docs" className="hover:text-white transition">
          Docs
        </Link>
        <span>/</span>
        <span className="text-white">Getting Started</span>
      </div>

      <h1 className="text-4xl font-bold mb-6">Getting Started</h1>
      <p className="text-zinc-400 text-lg mb-8">
        Go from zero to exploring your personal data in under 5 minutes.
      </p>

      <div className="space-y-12 text-zinc-300 leading-relaxed">
        {/* Step 1 */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            1. Download MemryLab
          </h2>
          <p className="mb-4">
            Head to the{" "}
            <a
              href="https://github.com/laadtushar/MemryLab/releases"
              className="text-violet-400 hover:text-violet-300 underline"
            >
              GitHub Releases
            </a>{" "}
            page and download the installer for your platform:
          </p>
          <ul className="list-disc list-inside space-y-2 text-zinc-400">
            <li>
              <strong className="text-white">Windows:</strong>{" "}
              <code className="px-2 py-1 rounded bg-zinc-900 text-sm">
                MemryLab_x.x.x_x64-setup.exe
              </code>
            </li>
            <li>
              <strong className="text-white">macOS:</strong>{" "}
              <code className="px-2 py-1 rounded bg-zinc-900 text-sm">
                MemryLab_x.x.x_universal.dmg
              </code>
            </li>
            <li>
              <strong className="text-white">Linux:</strong>{" "}
              <code className="px-2 py-1 rounded bg-zinc-900 text-sm">
                memrylab_x.x.x_amd64.deb
              </code>{" "}
              or{" "}
              <code className="px-2 py-1 rounded bg-zinc-900 text-sm">
                .AppImage
              </code>
            </li>
          </ul>
        </section>

        {/* Step 2 */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            2. Configure an AI Provider
          </h2>
          <p className="mb-4">
            MemryLab needs an LLM to analyze your data. Open{" "}
            <strong className="text-white">Settings</strong> and add an API key
            for any supported provider. We recommend starting with{" "}
            <strong className="text-white">Google Gemini</strong> (free tier,
            generous limits).
          </p>
          <pre className="bg-zinc-900 rounded-lg p-4 text-sm overflow-x-auto mb-4">
            <code>{`1. Open MemryLab → Settings → LLM Providers
2. Select "Gemini" from the provider dropdown
3. Paste your API key (get one at ai.google.dev)
4. Click "Test Connection" to verify`}</code>
          </pre>
          <p className="text-zinc-500 text-sm">
            See{" "}
            <Link
              href="/docs/ai-providers"
              className="text-violet-400 hover:text-violet-300 underline"
            >
              AI Providers
            </Link>{" "}
            for all 9 supported providers and free tier details.
          </p>
        </section>

        {/* Step 3 */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            3. Import Your Data
          </h2>
          <p className="mb-4">
            Click the <strong className="text-white">Import</strong> button and
            select a file, folder, or ZIP archive. MemryLab auto-detects the
            source format.
          </p>
          <pre className="bg-zinc-900 rounded-lg p-4 text-sm overflow-x-auto mb-4">
            <code>{`Supported formats:
• ZIP archives (Google Takeout, Facebook, etc.)
• Folders (Obsidian vaults, Markdown collections)
• JSON files (Twitter archive, Reddit export)
• Text/CSV files (WhatsApp, Telegram exports)
• ENEX files (Evernote)
• Day One JSON exports`}</code>
          </pre>
        </section>

        {/* Step 4 */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            4. Run Analysis
          </h2>
          <p className="mb-4">
            After importing, click{" "}
            <strong className="text-white">Run Analysis</strong> on the main
            dashboard. MemryLab runs an 8-stage pipeline:
          </p>
          <ol className="list-decimal list-inside space-y-2 text-zinc-400">
            <li>
              <strong className="text-white">Ingestion</strong> — Parse and
              normalize entries
            </li>
            <li>
              <strong className="text-white">Themes</strong> — Extract dominant
              themes
            </li>
            <li>
              <strong className="text-white">Sentiment</strong> — Analyze
              emotional tone
            </li>
            <li>
              <strong className="text-white">Beliefs</strong> — Identify stated
              beliefs and values
            </li>
            <li>
              <strong className="text-white">Entities</strong> — Extract people,
              places, concepts
            </li>
            <li>
              <strong className="text-white">Insights</strong> — Generate
              personal insights
            </li>
            <li>
              <strong className="text-white">Contradictions</strong> — Find
              evolving viewpoints
            </li>
            <li>
              <strong className="text-white">Narrative</strong> — Create
              evolution narratives
            </li>
          </ol>
        </section>

        {/* Step 5 */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            5. Explore
          </h2>
          <p>
            Use the <strong className="text-white">Timeline</strong> to browse
            entries over time, the{" "}
            <strong className="text-white">Knowledge Graph</strong> to see
            entity relationships, the{" "}
            <strong className="text-white">Evolution Explorer</strong> to track
            how themes changed, or{" "}
            <strong className="text-white">RAG Chat</strong> to ask questions
            about your own writing.
          </p>
        </section>
      </div>

      {/* Navigation */}
      <div className="flex items-center justify-between mt-16 pt-8 border-t border-zinc-800">
        <Link
          href="/docs"
          className="text-sm text-zinc-500 hover:text-white transition"
        >
          &larr; Docs Overview
        </Link>
        <Link
          href="/docs/installation"
          className="text-sm text-violet-400 hover:text-violet-300 transition"
        >
          Installation &rarr;
        </Link>
      </div>
    </div>
  );
}
