import type { Metadata } from "next";
import Link from "next/link";
import { notFound } from "next/navigation";
import FamilyFooter from "@/components/FamilyFooter";
import { JsonLd } from "@/lib/jsonld";
import { pageMetadata } from "@/lib/seo";
import { AUTHOR, LATEST_RELEASE_URL, PUBLISHER, REPO_URL, SITE_URL } from "@/lib/site";
import { articles } from "../articles";

export function generateStaticParams() {
  return articles.map((a) => ({ slug: a.slug }));
}

export async function generateMetadata({
  params,
}: {
  params: Promise<{ slug: string }>;
}): Promise<Metadata> {
  const { slug } = await params;
  const article = articles.find((a) => a.slug === slug);
  if (!article) return {};
  return pageMetadata({
    title: article.metaTitle,
    description: article.description,
    path: `/blog/${article.slug}`,
    type: "article",
    publishedTime: article.date,
    authors: [article.author],
  });
}

function escapeHtml(code: string) {
  return code.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function renderMarkdown(md: string) {
  // Simple markdown to HTML (tables, headers, bold, italic, code, links, lists)
  // Fenced code blocks are lifted out first so blank lines inside them do not split the block.
  const fences: string[] = [];
  const withoutFences = md.replace(/```[^\n]*\n([\s\S]*?)```/g, (_match, code: string) => {
    fences.push(code.replace(/\n$/, ""));
    return `@@FENCE${fences.length - 1}@@`;
  });

  return withoutFences
    .split("\n\n")
    .map((block, i) => {
      const trimmed = block.trim();

      // Headers
      if (trimmed.startsWith("### ")) return `<h3 class="text-xl font-semibold text-white mt-8 mb-3">${trimmed.slice(4)}</h3>`;
      if (trimmed.startsWith("## ")) return `<h2 class="text-2xl font-semibold text-white mt-10 mb-4">${trimmed.slice(3)}</h2>`;
      if (trimmed.startsWith("# ")) return `<h1 class="text-3xl font-bold text-white mt-6 mb-6">${trimmed.slice(2)}</h1>`;

      // Code blocks
      const fence = trimmed.match(/^@@FENCE(\d+)@@$/);
      if (fence) {
        return `<pre class="bg-zinc-900 rounded-lg p-4 text-sm overflow-x-auto my-4 border border-zinc-800"><code>${escapeHtml(fences[Number(fence[1])])}</code></pre>`;
      }

      // Tables
      if (trimmed.includes("|") && trimmed.includes("---")) {
        const rows = trimmed.split("\n").filter((r) => !r.match(/^\|[\s-|]+\|$/));
        const header = rows[0];
        const body = rows.slice(1);
        const headerCells = header.split("|").filter(Boolean).map((c) => c.trim());
        const html = `<div class="overflow-x-auto my-4"><table class="w-full text-sm"><thead><tr class="border-b border-zinc-800">${headerCells.map((c) => `<th class="py-2 pr-4 text-left text-zinc-400 font-medium">${c}</th>`).join("")}</tr></thead><tbody>${body.map((row) => {
          const cells = row.split("|").filter(Boolean).map((c) => c.trim());
          return `<tr class="border-b border-zinc-800/50">${cells.map((c) => `<td class="py-2 pr-4">${inlineFormat(c)}</td>`).join("")}</tr>`;
        }).join("")}</tbody></table></div>`;
        return html;
      }

      // Unordered lists
      if (trimmed.startsWith("- ")) {
        const items = trimmed.split("\n").map((l) => l.replace(/^- /, ""));
        return `<ul class="list-disc list-inside space-y-1 text-zinc-300 my-3">${items.map((l) => `<li>${inlineFormat(l)}</li>`).join("")}</ul>`;
      }

      // Ordered lists
      if (/^\d+\./.test(trimmed)) {
        const items = trimmed.split("\n").map((l) => l.replace(/^\d+\.\s*/, ""));
        return `<ol class="list-decimal list-inside space-y-1 text-zinc-300 my-3">${items.map((l) => `<li>${inlineFormat(l)}</li>`).join("")}</ol>`;
      }

      // Paragraph
      return `<p class="text-zinc-300 leading-relaxed my-3">${inlineFormat(trimmed)}</p>`;
    })
    .join("\n");
}

function inlineFormat(text: string): string {
  return text
    .replace(/\*\*(.+?)\*\*/g, '<strong class="text-white font-semibold">$1</strong>')
    .replace(/\*(.+?)\*/g, '<em class="text-zinc-200 italic">$1</em>')
    .replace(/`([^`]+)`/g, '<code class="bg-zinc-900 px-1.5 py-0.5 rounded text-violet-400 text-sm">$1</code>')
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, (_match, label: string, href: string) => {
      // Links to this site stay in the same tab; everything else opens in a new one.
      const internal = href.startsWith("/") || href.startsWith(SITE_URL);
      const attrs = internal ? "" : ' target="_blank" rel="noopener noreferrer"';
      return `<a href="${href}" class="text-violet-400 hover:text-violet-300 underline"${attrs}>${label}</a>`;
    });
}

export default async function BlogPost({
  params,
}: {
  params: Promise<{ slug: string }>;
}) {
  const { slug } = await params;
  const article = articles.find((a) => a.slug === slug);
  if (!article) notFound();

  const htmlContent = renderMarkdown(article.content);

  const blogPosting = {
    "@context": "https://schema.org",
    "@type": "BlogPosting",
    headline: article.title,
    description: article.description,
    url: `${SITE_URL}/blog/${article.slug}`,
    mainEntityOfPage: `${SITE_URL}/blog/${article.slug}`,
    datePublished: article.date,
    author: AUTHOR,
    publisher: PUBLISHER,
    keywords: article.tags.join(", "),
  };

  return (
    <div className="min-h-screen bg-black text-white">
      <JsonLd data={blogPosting} />
      {/* Header */}
      <header className="border-b border-zinc-800">
        <div className="max-w-3xl mx-auto px-6 py-4 flex items-center justify-between">
          <Link href="/" className="text-xl font-bold">MemryLab</Link>
          <nav className="flex items-center gap-6 text-sm text-zinc-400">
            <Link href="/docs" className="hover:text-white transition">Docs</Link>
            <Link href="/blog" className="hover:text-white transition">Blog</Link>
            <a href={REPO_URL} target="_blank" rel="noopener noreferrer" className="hover:text-white transition">GitHub</a>
          </nav>
        </div>
      </header>

      <main className="max-w-3xl mx-auto px-6 py-16">
        {/* Breadcrumb */}
        <div className="flex items-center gap-2 text-sm text-zinc-500 mb-8">
          <Link href="/blog" className="hover:text-white transition">Blog</Link>
          <span>/</span>
          <span className="text-white truncate">{article.title}</span>
        </div>

        {/* Meta */}
        <div className="flex items-center gap-3 text-sm text-zinc-500 mb-6">
          <span className="text-zinc-300 font-medium">{article.author}</span>
          <span>&middot;</span>
          <time dateTime={article.date}>{article.date}</time>
          <span>&middot;</span>
          <span>{article.readTime}</span>
          {article.tags.map((tag) => (
            <span key={tag} className="px-2 py-0.5 rounded bg-violet-500/10 text-violet-400 text-xs">
              {tag}
            </span>
          ))}
        </div>

        {/* Content */}
        <article
          className="prose prose-invert max-w-none"
          dangerouslySetInnerHTML={{ __html: htmlContent }}
        />

        {/* CTA */}
        <div className="mt-16 rounded-xl border border-violet-500/20 bg-violet-500/5 p-8 text-center">
          <h3 className="text-xl font-semibold mb-2">Ready to explore your data?</h3>
          <p className="text-zinc-400 mb-4">Download MemryLab — free, open source, privacy-first.</p>
          <a
            href={LATEST_RELEASE_URL}
            className="inline-flex items-center gap-2 rounded-lg bg-violet-600 px-6 py-3 text-white font-medium hover:bg-violet-500 transition-colors"
          >
            Download MemryLab
          </a>
        </div>

        {/* Related articles */}
        <div className="mt-16 border-t border-zinc-800 pt-8">
          <h3 className="text-lg font-semibold mb-4">More from the blog</h3>
          <div className="grid gap-4 sm:grid-cols-2">
            {articles
              .filter((a) => a.slug !== slug)
              .slice(0, 2)
              .map((a) => (
                <Link
                  key={a.slug}
                  href={`/blog/${a.slug}`}
                  className="rounded-lg border border-zinc-800 p-4 hover:border-violet-500/40 transition-colors"
                >
                  <p className="text-xs text-zinc-500 mb-1">{a.date}</p>
                  <p className="font-medium text-sm">{a.title}</p>
                </Link>
              ))}
          </div>
        </div>

        <FamilyFooter className="mt-16 pt-6 border-t border-zinc-800" />
      </main>
    </div>
  );
}
