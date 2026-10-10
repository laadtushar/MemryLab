import Link from "next/link";
import { pageMetadata } from "@/lib/seo";
import { DOWNLOADS, LATEST_RELEASE_URL, LATEST_VERSION, WINDOWS_INSTALLER_SIZE } from "@/lib/site";

export const metadata = pageMetadata({
  title: `Install MemryLab ${LATEST_VERSION} on Windows, macOS and Linux`,
  description:
    "Download the MemryLab installer for Windows, macOS or Linux, or build it from source. Includes file names, where your data is stored and how API keys are kept.",
  path: "/docs/installation",
});

const downloads = [
  { platform: "Windows", format: "NSIS installer (x64)", ...DOWNLOADS.windowsExe },
  { platform: "Windows", format: "MSI (x64)", ...DOWNLOADS.windowsMsi },
  { platform: "macOS", format: "Universal DMG (Intel and Apple Silicon)", ...DOWNLOADS.macosDmg },
  { platform: "Linux", format: "Debian/Ubuntu package", ...DOWNLOADS.linuxDeb },
  { platform: "Linux", format: "AppImage", ...DOWNLOADS.linuxAppImage },
  { platform: "Linux", format: "Fedora/RPM package", ...DOWNLOADS.linuxRpm },
];

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
            MemryLab has minimal system requirements. The Windows installer is
            only {WINDOWS_INSTALLER_SIZE} thanks to Tauri.
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

        {/* Downloads */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            Downloads (v{LATEST_VERSION})
          </h2>
          <p className="mb-4">
            These are the installers attached to the{" "}
            <a
              href={LATEST_RELEASE_URL}
              className="text-violet-400 hover:text-violet-300 underline"
            >
              latest GitHub release
            </a>
            .
          </p>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-zinc-800 text-left">
                  <th className="py-3 pr-4 text-zinc-400 font-medium">Platform</th>
                  <th className="py-3 pr-4 text-zinc-400 font-medium">Format</th>
                  <th className="py-3 text-zinc-400 font-medium">File</th>
                </tr>
              </thead>
              <tbody>
                {downloads.map((d) => (
                  <tr key={d.file} className="border-b border-zinc-800/50">
                    <td className="py-3 pr-4 text-white font-medium">{d.platform}</td>
                    <td className="py-3 pr-4 text-zinc-400">{d.format}</td>
                    <td className="py-3">
                      <a href={d.url} className="text-violet-400 hover:text-violet-300 underline break-all">
                        {d.file}
                      </a>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        {/* Windows */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">Windows</h2>
          <pre className="bg-zinc-900 rounded-lg p-4 text-sm overflow-x-auto mb-4">
            <code>{`# Download from GitHub Releases
# Run the .exe installer — it handles everything
${DOWNLOADS.windowsExe.file}

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
sudo dpkg -i ${DOWNLOADS.linuxDeb.file}

# Or use the AppImage
chmod +x ${DOWNLOADS.linuxAppImage.file}
./${DOWNLOADS.linuxAppImage.file}

# Fedora/RPM-based distributions
sudo dnf install ./${DOWNLOADS.linuxRpm.file}

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
# - Node.js 20+ (nodejs.org)
# - Platform-specific Tauri dependencies
#   See: https://v2.tauri.app/start/prerequisites/

# Clone and build
git clone https://github.com/laadtushar/MemryLab.git
cd MemryLab
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
            All data is stored locally. Local by default: your data stays on
            your device unless you choose a cloud model, in which case the text
            it analyses is sent to that provider&apos;s API.
          </p>
          <pre className="bg-zinc-900 rounded-lg p-4 text-sm overflow-x-auto">
            <code>{`# Database location
Windows:  %APPDATA%/com.memrylab.app/memory_palace.db
macOS:    ~/Library/Application Support/com.memrylab.app/memory_palace.db
Linux:    ~/.local/share/com.memrylab.app/memory_palace.db

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
