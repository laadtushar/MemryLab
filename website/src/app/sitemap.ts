import type { MetadataRoute } from "next";
import { articles } from "./blog/articles";
import { SITE_URL } from "@/lib/site";

// Public, indexable pages only. `lastModified` is set only where the repo has a real
// date: blog posts carry their publish date. The other pages have no recorded date, so
// the field is left out rather than invented.
const staticPaths = [
  "/",
  "/docs",
  "/docs/getting-started",
  "/docs/installation",
  "/docs/import-sources",
  "/docs/ai-providers",
  "/docs/architecture",
  "/docs/contributing",
];

export default function sitemap(): MetadataRoute.Sitemap {
  const latestPost = articles.map((a) => a.date).sort().at(-1);
  return [
    ...staticPaths.map((path) => ({ url: `${SITE_URL}${path === "/" ? "" : path}` })),
    { url: `${SITE_URL}/blog`, ...(latestPost ? { lastModified: latestPost } : {}) },
    ...articles.map((a) => ({ url: `${SITE_URL}/blog/${a.slug}`, lastModified: a.date })),
  ];
}
