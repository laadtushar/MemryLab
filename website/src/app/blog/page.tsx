import Link from "next/link";
import FamilyFooter from "@/components/FamilyFooter";
import { pageMetadata } from "@/lib/seo";
import { REPO_URL } from "@/lib/site";
import { articles } from "./articles";

export const metadata = pageMetadata({
  title: "MemryLab Blog: Personal Knowledge, Memory and Private AI",
  description:
    "Essays and guides on tracking how your beliefs change, running local LLMs with Ollama and exporting your data from 30+ platforms, by Tushar Laad.",
  path: "/blog",
});

export default function BlogIndex() {
  return (
    <div className="min-h-screen bg-black text-white">
      {/* Header */}
      <header className="border-b border-zinc-800">
        <div className="max-w-5xl mx-auto px-6 py-4 flex items-center justify-between">
          <Link href="/" className="text-xl font-bold">
            MemryLab
          </Link>
          <nav className="flex items-center gap-6 text-sm text-zinc-400">
            <Link href="/docs" className="hover:text-white transition">Docs</Link>
            <Link href="/blog" className="text-white font-medium">Blog</Link>
            <a href={REPO_URL} target="_blank" rel="noopener noreferrer" className="hover:text-white transition">GitHub</a>
          </nav>
        </div>
      </header>

      <main className="max-w-5xl mx-auto px-6 py-16">
        <h1 className="text-4xl font-bold mb-4">Blog</h1>
        <p className="text-zinc-400 text-lg mb-12">
          Thoughts on personal knowledge, belief evolution, and building privacy-first AI tools.
        </p>

        <div className="grid gap-8 md:grid-cols-2">
          {articles.map((article) => (
            <Link
              key={article.slug}
              href={`/blog/${article.slug}`}
              className="group rounded-xl border border-zinc-800 bg-zinc-900/50 p-6 hover:border-violet-500/40 hover:bg-zinc-900 transition-all"
            >
              <div className="flex items-center gap-2 text-xs text-zinc-500 mb-3">
                <time dateTime={article.date}>{article.date}</time>
                <span>&middot;</span>
                <span>{article.readTime}</span>
                {article.tags.map((tag) => (
                  <span key={tag} className="px-2 py-0.5 rounded bg-violet-500/10 text-violet-400">
                    {tag}
                  </span>
                ))}
              </div>
              <h2 className="text-xl font-semibold mb-2 group-hover:text-violet-400 transition-colors">
                {article.title}
              </h2>
              <p className="text-zinc-400 text-sm leading-relaxed">
                {article.excerpt}
              </p>
              <p className="text-sm text-zinc-600 mt-3">By {article.author}</p>
            </Link>
          ))}
        </div>
        <FamilyFooter className="mt-16 pt-6 border-t border-zinc-800" />
      </main>
    </div>
  );
}
