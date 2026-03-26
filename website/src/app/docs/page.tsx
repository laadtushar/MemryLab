import Link from "next/link";

const sections = [
  {
    href: "/docs/getting-started",
    title: "Getting Started",
    desc: "Download MemryLab, run your first import, and analyze your data in under 5 minutes.",
  },
  {
    href: "/docs/installation",
    title: "Installation",
    desc: "Platform-specific instructions for Windows, macOS, and Linux. Build from source.",
  },
  {
    href: "/docs/import-sources",
    title: "Import Sources",
    desc: "Complete guide to all 30+ supported platforms with data export instructions.",
  },
  {
    href: "/docs/ai-providers",
    title: "AI Providers",
    desc: "Set up Gemini, Groq, OpenRouter, Cerebras, Mistral, SambaNova, Cohere, or OpenAI.",
  },
  {
    href: "/docs/architecture",
    title: "Architecture",
    desc: "Hexagonal architecture, Tauri 2.0, Rust backend, and data flow overview.",
  },
  {
    href: "/docs/contributing",
    title: "Contributing",
    desc: "Add new import adapters, fix bugs, or improve the UI. Contribution guide.",
  },
];

export default function DocsPage() {
  return (
    <div>
      <h1 className="text-4xl font-bold mb-4">Documentation</h1>
      <p className="text-zinc-400 text-lg mb-12">
        Everything you need to get started with MemryLab — from installation to
        building your own import adapters.
      </p>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {sections.map((s) => (
          <Link
            key={s.href}
            href={s.href}
            className="glass rounded-xl p-6 hover:border-violet-500/30 transition group block"
          >
            <h2 className="text-lg font-semibold mb-2 group-hover:text-violet-400 transition">
              {s.title}
            </h2>
            <p className="text-sm text-zinc-400">{s.desc}</p>
          </Link>
        ))}
      </div>
    </div>
  );
}
