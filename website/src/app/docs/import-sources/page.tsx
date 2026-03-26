import Link from "next/link";

const sources = [
  {
    name: "Google Takeout",
    format: "ZIP",
    notes: "Gmail, Chat, Keep, YouTube, Chrome, Maps, and more",
    link: "https://takeout.google.com",
  },
  {
    name: "Facebook",
    format: "ZIP (JSON)",
    notes: "Posts, messages, comments, reactions",
    link: "https://www.facebook.com/dyi",
  },
  {
    name: "Instagram",
    format: "ZIP (JSON)",
    notes: "Posts, stories, messages, comments",
    link: "https://www.instagram.com/download/request/",
  },
  {
    name: "Twitter / X",
    format: "ZIP (JSON)",
    notes: "Tweets, DMs, likes, bookmarks",
    link: "https://twitter.com/settings/download_your_data",
  },
  {
    name: "Reddit",
    format: "ZIP (CSV)",
    notes: "Posts, comments, saved items, messages",
    link: "https://www.reddit.com/settings/data-request",
  },
  {
    name: "WhatsApp",
    format: "TXT / ZIP",
    notes: "Chat exports from individual or group conversations",
    link: "",
  },
  {
    name: "Telegram",
    format: "JSON",
    notes: "Exported via Telegram Desktop (JSON format recommended)",
    link: "",
  },
  {
    name: "Discord",
    format: "ZIP",
    notes: "Messages via Discord data request",
    link: "https://discord.com/developers/docs",
  },
  {
    name: "Slack",
    format: "ZIP (JSON)",
    notes: "Workspace export — messages, channels, threads",
    link: "",
  },
  {
    name: "LinkedIn",
    format: "ZIP (CSV)",
    notes: "Messages, connections, profile data",
    link: "https://www.linkedin.com/mypreferences/d/download-my-data",
  },
  {
    name: "Obsidian",
    format: "Folder",
    notes: "Point to your vault folder — reads all .md files with frontmatter",
    link: "",
  },
  {
    name: "Notion",
    format: "ZIP (Markdown)",
    notes: "Export workspace as Markdown — pages, databases",
    link: "",
  },
  {
    name: "Evernote",
    format: "ENEX",
    notes: "Export notebooks as .enex XML files",
    link: "",
  },
  {
    name: "Day One",
    format: "JSON",
    notes: "Export via Day One app (JSON format)",
    link: "",
  },
  {
    name: "Markdown",
    format: "Folder / File",
    notes: "Any collection of .md or .txt files",
    link: "",
  },
  {
    name: "Spotify",
    format: "ZIP (JSON)",
    notes: "Streaming history, playlists, search queries",
    link: "https://www.spotify.com/account/privacy/",
  },
  {
    name: "YouTube",
    format: "ZIP (JSON)",
    notes: "Watch history, comments, subscriptions (via Google Takeout)",
    link: "https://takeout.google.com",
  },
  {
    name: "TikTok",
    format: "ZIP (JSON)",
    notes: "Videos, comments, messages, browsing history",
    link: "https://www.tiktok.com/setting/download-your-data",
  },
  {
    name: "Snapchat",
    format: "ZIP (JSON)",
    notes: "Memories, chat history, snap history",
    link: "https://accounts.snapchat.com/accounts/downloadmydata",
  },
  {
    name: "Bluesky",
    format: "JSON (CAR)",
    notes: "Posts, likes, follows via AT Protocol repo export",
    link: "",
  },
  {
    name: "Mastodon",
    format: "CSV / JSON",
    notes: "Archive request from your instance settings",
    link: "",
  },
  {
    name: "Substack",
    format: "ZIP",
    notes: "Posts, comments, subscriber notes",
    link: "",
  },
  {
    name: "Medium",
    format: "ZIP (HTML)",
    notes: "Posts, responses, highlights",
    link: "https://medium.com/me/settings/security",
  },
  {
    name: "Tumblr",
    format: "ZIP",
    notes: "Posts, messages, likes",
    link: "",
  },
  {
    name: "Pinterest",
    format: "ZIP",
    notes: "Pins, boards, search history",
    link: "",
  },
  {
    name: "Apple",
    format: "ZIP",
    notes: "Apple privacy data request — iMessage, Notes, Health",
    link: "https://privacy.apple.com",
  },
  {
    name: "Amazon",
    format: "ZIP",
    notes: "Order history, reviews, searches",
    link: "https://www.amazon.com/gp/privacycentral/dsar/preview.html",
  },
  {
    name: "Microsoft",
    format: "ZIP",
    notes: "Outlook, Teams, OneDrive activity via privacy dashboard",
    link: "https://account.microsoft.com/privacy",
  },
  {
    name: "Signal",
    format: "Backup",
    notes: "Encrypted backup file from Signal settings",
    link: "",
  },
];

export default function ImportSourcesPage() {
  return (
    <div>
      <div className="flex items-center gap-2 text-sm text-zinc-500 mb-8">
        <Link href="/docs" className="hover:text-white transition">
          Docs
        </Link>
        <span>/</span>
        <span className="text-white">Import Sources</span>
      </div>

      <h1 className="text-4xl font-bold mb-6">Import Sources</h1>
      <p className="text-zinc-400 text-lg mb-8">
        MemryLab supports 30+ platforms. Just drop a file, folder, or ZIP — the
        format is auto-detected.
      </p>

      <div className="space-y-8 text-zinc-300 leading-relaxed">
        {/* How to import */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            How to Import
          </h2>
          <ol className="list-decimal list-inside space-y-2 text-zinc-400">
            <li>
              Click <strong className="text-white">Import</strong> on the
              sidebar
            </li>
            <li>
              Select a file, folder, or ZIP archive from your computer
            </li>
            <li>
              MemryLab auto-detects the source type and parses entries
            </li>
            <li>
              Review the import summary and confirm
            </li>
          </ol>
          <p className="mt-4 text-zinc-500 text-sm">
            ZIP files are extracted in-memory — no temporary files are written to
            disk. The original archive is never modified.
          </p>
        </section>

        {/* Table */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            Supported Sources
          </h2>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-zinc-800 text-left">
                  <th className="py-3 pr-4 text-zinc-400 font-medium">
                    Source
                  </th>
                  <th className="py-3 pr-4 text-zinc-400 font-medium">
                    Format
                  </th>
                  <th className="py-3 pr-4 text-zinc-400 font-medium">
                    Notes
                  </th>
                  <th className="py-3 text-zinc-400 font-medium">
                    Export Link
                  </th>
                </tr>
              </thead>
              <tbody>
                {sources.map((s, i) => (
                  <tr
                    key={i}
                    className="border-b border-zinc-800/50 hover:bg-zinc-900/30"
                  >
                    <td className="py-3 pr-4 text-white font-medium">
                      {s.name}
                    </td>
                    <td className="py-3 pr-4">
                      <code className="px-2 py-0.5 rounded bg-zinc-900 text-xs">
                        {s.format}
                      </code>
                    </td>
                    <td className="py-3 pr-4 text-zinc-400">{s.notes}</td>
                    <td className="py-3">
                      {s.link ? (
                        <a
                          href={s.link}
                          target="_blank"
                          className="text-violet-400 hover:text-violet-300 text-xs underline"
                        >
                          Export
                        </a>
                      ) : (
                        <span className="text-zinc-600 text-xs">In-app</span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        {/* Tips */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">Tips</h2>
          <ul className="list-disc list-inside space-y-2 text-zinc-400">
            <li>
              <strong className="text-white">Google Takeout:</strong> Select
              only the products you need to reduce download size and processing
              time.
            </li>
            <li>
              <strong className="text-white">WhatsApp:</strong> Export chats
              from within the app (three-dot menu &gt; More &gt; Export chat).
              Choose &quot;Without media&quot; for faster processing.
            </li>
            <li>
              <strong className="text-white">Twitter/X:</strong> Request your
              archive from Settings &gt; Your account &gt; Download an archive.
              It may take 24-48 hours.
            </li>
            <li>
              <strong className="text-white">Obsidian:</strong> Point directly
              to your vault folder. MemryLab respects{" "}
              <code className="px-1 py-0.5 rounded bg-zinc-900 text-xs">
                .obsidianignore
              </code>{" "}
              patterns.
            </li>
          </ul>
        </section>
      </div>

      <div className="flex items-center justify-between mt-16 pt-8 border-t border-zinc-800">
        <Link
          href="/docs/installation"
          className="text-sm text-zinc-500 hover:text-white transition"
        >
          &larr; Installation
        </Link>
        <Link
          href="/docs/ai-providers"
          className="text-sm text-violet-400 hover:text-violet-300 transition"
        >
          AI Providers &rarr;
        </Link>
      </div>
    </div>
  );
}
