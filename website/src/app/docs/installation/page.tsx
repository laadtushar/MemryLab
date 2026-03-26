import Link from "next/link";

export default function InstallationPage() {
  return (
    <div>
      <div className="flex items-center gap-2 text-sm text-zinc-500 mb-8">
        <Link href="/docs" className="hover:text-white transition">
          Docs
        </Link>
        <span>/</span>
        <span className="text-white">Installation</span>
      </div>

      <h1 className="text-4xl font-bold mb-6">Installation</h1>
      <p className="text-zinc-400 text-lg mb-8">
        MemryLab is a desktop application built with Tauri 2.0. Download the
        pre-built installer or build from source.
      </p>

      <div className="space-y-12 text-zinc-300 leading-relaxed">
        {/* Prerequisites */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            Prerequisites
          </h2>
          <p className="mb-4">
            MemryLab has minimal system requirements. The installer is under 5MB
            thanks to Tauri.
          </p>
          <ul className="list-disc list-inside space-y-2 text-zinc-400">
            <li>
              <strong className="text-white">Windows 10+</strong> (x64)
            </li>
            <li>
              <strong className="text-white">macOS 11+</strong> (Intel or Apple
              Silicon)
            </li>
            <li>
              <strong className="text-white">Linux</strong> — Ubuntu 20.04+,
              Fedora 36+, or equivalent with WebKit2GTK
            </li>
            <li>
              <strong className="text-white">Internet</strong> — Required only
              for AI API calls, not for the app itself
            </li>
          </ul>
        </section>

        {/* Windows */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">Windows</h2>
          <pre className="bg-zinc-900 rounded-lg p-4 text-sm overflow-x-auto mb-4">
            <code>{`# Download from GitHub Releases
# Run the .exe installer — it handles everything
MemryLab_0.1.0_x64-setup.exe

# Or use winget (coming soon)
winget install memrylab`}</code>
          </pre>
          <p className="text-zinc-500 text-sm">
            Windows may show a SmartScreen warning since the binary is not
            code-signed yet. Click &quot;More info&quot; then &quot;Run
            anyway&quot;.
          </p>
        </section>

        {/* macOS */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">macOS</h2>
          <pre className="bg-zinc-900 rounded-lg p-4 text-sm overflow-x-auto mb-4">
            <code>{`# Download the .dmg from GitHub Releases
# Drag MemryLab.app to Applications

# Or use Homebrew (coming soon)
brew install --cask memrylab`}</code>
          </pre>
          <p className="text-zinc-500 text-sm">
            On first launch, macOS may block the app. Go to System Settings
            &gt; Privacy &amp; Security and click &quot;Open Anyway&quot;.
          </p>
        </section>

        {/* Linux */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">Linux</h2>
          <pre className="bg-zinc-900 rounded-lg p-4 text-sm overflow-x-auto mb-4">
            <code>{`# Debian/Ubuntu
sudo dpkg -i memrylab_0.1.0_amd64.deb

# Or use the AppImage
chmod +x MemryLab_0.1.0_amd64.AppImage
./MemryLab_0.1.0_amd64.AppImage

# Dependencies (if using .deb)
sudo apt install libwebkit2gtk-4.1-0 libjavascriptcoregtk-4.1-0`}</code>
          </pre>
        </section>

        {/* Build from source */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            Build from Source
          </h2>
          <p className="mb-4">
            MemryLab uses Tauri 2.0 with a Rust backend and React + TypeScript
            frontend.
          </p>
          <pre className="bg-zinc-900 rounded-lg p-4 text-sm overflow-x-auto mb-4">
            <code>{`# Prerequisites
# - Rust 1.75+ (rustup.rs)
# - Node.js 18+ (nodejs.org)
# - Platform-specific Tauri dependencies
#   See: https://v2.tauri.app/start/prerequisites/

# Clone and build
git clone https://github.com/laadtushar/MemryLab.git
cd MemPalace
npm install

# Development mode
npm run tauri dev

# Production build
npm run tauri build`}</code>
          </pre>
          <p className="text-zinc-500 text-sm">
            The production build creates platform-specific installers in{" "}
            <code className="px-2 py-1 rounded bg-zinc-900 text-sm">
              src-tauri/target/release/bundle/
            </code>
          </p>
        </section>

        {/* Data location */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            Data Storage
          </h2>
          <p className="mb-4">
            All data is stored locally. Nothing is sent to external servers
            (except LLM API calls).
          </p>
          <pre className="bg-zinc-900 rounded-lg p-4 text-sm overflow-x-auto">
            <code>{`# Database location
Windows:  %APPDATA%/com.memrylab.app/mempalace.db
macOS:    ~/Library/Application Support/com.memrylab.app/mempalace.db
Linux:    ~/.local/share/com.memrylab.app/mempalace.db

# API keys are stored in the OS keychain
Windows:  Windows Credential Manager
macOS:    Keychain Access
Linux:    libsecret (GNOME Keyring / KWallet)`}</code>
          </pre>
        </section>
      </div>

      <div className="flex items-center justify-between mt-16 pt-8 border-t border-zinc-800">
        <Link
          href="/docs/getting-started"
          className="text-sm text-zinc-500 hover:text-white transition"
        >
          &larr; Getting Started
        </Link>
        <Link
          href="/docs/import-sources"
          className="text-sm text-violet-400 hover:text-violet-300 transition"
        >
          Import Sources &rarr;
        </Link>
      </div>
    </div>
  );
}
