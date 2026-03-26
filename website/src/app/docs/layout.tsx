import Link from "next/link";

const sidebarItems = [
  { href: "/docs", label: "Overview" },
  { href: "/docs/getting-started", label: "Getting Started" },
  { href: "/docs/installation", label: "Installation" },
  { href: "/docs/import-sources", label: "Import Sources" },
  { href: "/docs/ai-providers", label: "AI Providers" },
  { href: "/docs/architecture", label: "Architecture" },
  { href: "/docs/contributing", label: "Contributing" },
];

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="min-h-screen bg-black">
      {/* Top nav */}
      <nav className="fixed top-0 left-0 right-0 z-50 glass">
        <div className="max-w-7xl mx-auto px-6 py-4 flex items-center justify-between">
          <Link href="/" className="flex items-center gap-2">
            <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-violet-500 to-blue-500 flex items-center justify-center text-white font-bold text-sm">
              M
            </div>
            <span className="text-lg font-bold">MemryLab</span>
          </Link>
          <div className="flex items-center gap-6 text-sm text-zinc-400">
            <Link href="/" className="hover:text-white transition">
              Home
            </Link>
            <Link href="/docs" className="text-white">
              Docs
            </Link>
            <a
              href="https://github.com/laadtushar/MemPalace"
              target="_blank"
              className="hover:text-white transition"
            >
              GitHub
            </a>
            <a
              href="https://github.com/laadtushar/MemPalace/releases"
              className="px-4 py-2 rounded-full bg-violet-600 text-white text-sm font-medium hover:bg-violet-500 transition"
            >
              Download
            </a>
          </div>
        </div>
      </nav>

      <div className="flex pt-16">
        {/* Sidebar */}
        <aside className="fixed left-0 top-16 bottom-0 w-64 border-r border-zinc-800 p-6 overflow-y-auto">
          <div className="text-xs font-semibold text-zinc-500 uppercase tracking-wider mb-4">
            Documentation
          </div>
          <ul className="space-y-1">
            {sidebarItems.map((item) => (
              <li key={item.href}>
                <Link
                  href={item.href}
                  className="block px-3 py-2 rounded-lg text-sm text-zinc-400 hover:text-white hover:bg-zinc-800/50 transition"
                >
                  {item.label}
                </Link>
              </li>
            ))}
          </ul>
        </aside>

        {/* Main content */}
        <main className="ml-64 flex-1 min-h-screen px-12 py-12 max-w-4xl">
          {children}
        </main>
      </div>
    </div>
  );
}
