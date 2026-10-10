import type { Metadata } from "next";
import { OG_IMAGE_ALT, SITE_NAME } from "./site";

// The image rendered by app/opengraph-image.tsx. Pages that set their own `openGraph` do not
// inherit the layout's file-based image, so it is listed explicitly here.
const OG_IMAGE = { url: "/opengraph-image", width: 1200, height: 630, alt: OG_IMAGE_ALT };

type PageSeo = {
  /** Full page title (about 50-60 characters). Used as-is, no suffix is appended. */
  title: string;
  /** Meta description (about 140-160 characters). */
  description: string;
  /** Canonical path, starting with "/" ("/" for the home page). */
  path: string;
  type?: "website" | "article";
  publishedTime?: string;
  modifiedTime?: string;
  authors?: string[];
};

/**
 * Builds the per-page metadata: unique title and description, a self-referencing
 * canonical, and matching Open Graph and Twitter tags. A page that sets its own
 * `openGraph` replaces the layout's whole object in Next.js, so everything is repeated here.
 */
export function pageMetadata({ title, description, path, type = "website", publishedTime, modifiedTime, authors }: PageSeo): Metadata {
  return {
    title: { absolute: title },
    description,
    alternates: { canonical: path },
    openGraph: {
      title,
      description,
      url: path,
      siteName: SITE_NAME,
      type,
      locale: "en_US",
      images: [OG_IMAGE],
      ...(type === "article" ? { publishedTime, ...(modifiedTime ? { modifiedTime } : {}), authors } : {}),
    },
    twitter: {
      card: "summary_large_image",
      title,
      description,
      images: [{ url: OG_IMAGE.url, alt: OG_IMAGE.alt }],
    },
  };
}
