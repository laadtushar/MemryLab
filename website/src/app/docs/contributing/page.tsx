import Link from "next/link";

export default function ContributingPage() {
  return (
    <div>
      <div className="flex items-center gap-2 text-sm text-zinc-500 mb-8">
        <Link href="/docs" className="hover:text-white transition">
          Docs
        </Link>
        <span>/</span>
        <span className="text-white">Contributing</span>
      </div>

      <h1 className="text-4xl font-bold mb-6">Contributing</h1>
      <p className="text-zinc-400 text-lg mb-8">
        MemryLab is open source and welcomes contributions. Here is how to get
        started.
      </p>

      <div className="space-y-12 text-zinc-300 leading-relaxed">
        {/* Getting Started */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            Development Setup
          </h2>
          <pre className="bg-zinc-900 rounded-lg p-4 text-sm overflow-x-auto mb-4">
            <code>{`# Prerequisites
# - Rust 1.75+ (https://rustup.rs)
# - Node.js 18+ (https://nodejs.org)
# - Tauri prerequisites for your OS
#   https://v2.tauri.app/start/prerequisites/

# Clone the repository
git clone https://github.com/laadtushar/MemryLab.git
cd MemryLab

# Install frontend dependencies
npm install

# Run in development mode (hot-reload for both Rust and React)
npm run tauri dev

# Run frontend only (useful for UI work)
npm run dev`}</code>
          </pre>
        </section>

        {/* Adding a Source Adapter */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            Adding a New Import Source
          </h2>
          <p className="mb-4">
            Each import source is a Rust module in{" "}
            <code className="px-2 py-0.5 rounded bg-zinc-900 text-xs">
              src-tauri/src/pipeline/ingestion/source_adapters/
            </code>
            . To add a new one:
          </p>
          <ol className="list-decimal list-inside space-y-3 text-zinc-400">
            <li>
              Create a new file like{" "}
              <code className="px-1 py-0.5 rounded bg-zinc-900 text-xs">
                myplatform.rs
              </code>
            </li>
            <li>
              Implement the{" "}
              <code className="px-1 py-0.5 rounded bg-zinc-900 text-xs">
                SourceAdapter
              </code>{" "}
              trait
            </li>
            <li>
              Register it in{" "}
              <code className="px-1 py-0.5 rounded bg-zinc-900 text-xs">
                mod.rs
              </code>
            </li>
            <li>Add detection logic so it auto-identifies the format</li>
            <li>Write tests with sample data</li>
          </ol>

          <pre className="bg-zinc-900 rounded-lg p-4 text-sm overflow-x-auto mt-4 mb-4">
            <code>{`// Example adapter skeleton
use crate::domain::models::common::Entry;
use crate::pipeline::ingestion::source_adapters::SourceAdapter;

pub struct MyPlatformAdapter;

impl SourceAdapter for MyPlatformAdapter {
    fn name(&self) -> &str {
        "myplatform"
    }

    fn can_handle(&self, path: &Path) -> bool {
        // Return true if this path looks like a MyPlatform export
        // Check for characteristic files/structure
    }

    fn parse(&self, path: &Path) -> Result<Vec<Entry>> {
        // Read and parse the export into Entry structs
        // Each entry needs: content, timestamp, source name
    }
}`}</code>
          </pre>
        </section>

        {/* Code Style */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            Code Style
          </h2>
          <ul className="list-disc list-inside space-y-2 text-zinc-400">
            <li>
              <strong className="text-white">Rust:</strong> Follow standard
              Rust conventions.{" "}
              <code className="px-1 py-0.5 rounded bg-zinc-900 text-xs">
                cargo fmt
              </code>{" "}
              and{" "}
              <code className="px-1 py-0.5 rounded bg-zinc-900 text-xs">
                cargo clippy
              </code>{" "}
              must pass.
            </li>
            <li>
              <strong className="text-white">TypeScript:</strong> Strict mode
              enabled. Use functional components with hooks. Prefer Zustand for
              state management.
            </li>
            <li>
              <strong className="text-white">CSS:</strong> Tailwind utility
              classes only. No custom CSS files except globals.
            </li>
            <li>
              <strong className="text-white">Commits:</strong> Use conventional
              commit messages (
              <code className="px-1 py-0.5 rounded bg-zinc-900 text-xs">
                feat:
              </code>
              ,{" "}
              <code className="px-1 py-0.5 rounded bg-zinc-900 text-xs">
                fix:
              </code>
              ,{" "}
              <code className="px-1 py-0.5 rounded bg-zinc-900 text-xs">
                docs:
              </code>
              ,{" "}
              <code className="px-1 py-0.5 rounded bg-zinc-900 text-xs">
                refactor:
              </code>
              ).
            </li>
          </ul>
        </section>

        {/* What to Work On */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            What to Work On
          </h2>
          <p className="mb-4">
            Check the{" "}
            <a
              href="https://github.com/laadtushar/MemryLab/issues"
              target="_blank"
              className="text-violet-400 hover:text-violet-300 underline"
            >
              GitHub Issues
            </a>{" "}
            for open tasks. Good first contributions include:
          </p>
          <ul className="list-disc list-inside space-y-2 text-zinc-400">
            <li>Adding new import source adapters</li>
            <li>Improving parsing accuracy for existing adapters</li>
            <li>UI improvements and accessibility</li>
            <li>Documentation and examples</li>
            <li>Performance optimizations for large datasets</li>
            <li>Adding tests for untested modules</li>
          </ul>
        </section>

        {/* PR Process */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            Pull Request Process
          </h2>
          <ol className="list-decimal list-inside space-y-2 text-zinc-400">
            <li>Fork the repository and create a feature branch</li>
            <li>
              Make your changes with clear, descriptive commits
            </li>
            <li>
              Ensure{" "}
              <code className="px-1 py-0.5 rounded bg-zinc-900 text-xs">
                cargo build
              </code>
              ,{" "}
              <code className="px-1 py-0.5 rounded bg-zinc-900 text-xs">
                cargo test
              </code>
              , and{" "}
              <code className="px-1 py-0.5 rounded bg-zinc-900 text-xs">
                cargo clippy
              </code>{" "}
              pass
            </li>
            <li>
              Open a PR with a clear description of what changed and why
            </li>
            <li>Respond to review feedback</li>
          </ol>
        </section>

        {/* License */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">License</h2>
          <p>
            MemryLab is released under the MIT License. By contributing, you
            agree that your contributions will be licensed under the same terms.
          </p>
        </section>
      </div>

      <div className="flex items-center justify-between mt-16 pt-8 border-t border-zinc-800">
        <Link
          href="/docs/architecture"
          className="text-sm text-zinc-500 hover:text-white transition"
        >
          &larr; Architecture
        </Link>
        <Link
          href="/docs"
          className="text-sm text-violet-400 hover:text-violet-300 transition"
        >
          Docs Overview &rarr;
        </Link>
      </div>
    </div>
  );
}
