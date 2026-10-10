import type { Metadata } from "next";
import "./globals.css";
import { SITE_DESCRIPTION, SITE_NAME, SITE_URL } from "@/lib/site";

// Pages set their own title, description, canonical and Open Graph tags through
// `pageMetadata` (lib/seo.ts). These are the site-wide defaults. The og:image comes
// from app/opengraph-image.tsx.
export const metadata: Metadata = {
  metadataBase: new URL(SITE_URL),
  title: {
    default: "MemryLab: Your Memory, Visualized | Local AI Memory App",
    template: `%s | ${SITE_NAME}`,
  },
  description: SITE_DESCRIPTION,
  applicationName: SITE_NAME,
  authors: [{ name: "Tushar Laad", url: "https://www.tusharlaad.com" }],
  icons: {
    icon: "/favicon.svg",
    apple: "/logo-icon.svg",
  },
  openGraph: {
    title: "MemryLab: Your Memory, Visualized | Local AI Memory App",
    description: SITE_DESCRIPTION,
    url: "/",
    siteName: SITE_NAME,
    type: "website",
    locale: "en_US",
  },
  twitter: {
    card: "summary_large_image",
    title: "MemryLab: Your Memory, Visualized | Local AI Memory App",
    description: SITE_DESCRIPTION,
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark">
      <head>
        {/* The page animations start at opacity 0 and need JavaScript to fade in, so without it show the content as rendered. */}
        <noscript>
          <style>{`[style*="opacity:0;"],[style$="opacity:0"]{opacity:1!important;transform:none!important}`}</style>
        </noscript>
      </head>
      <body className="bg-black text-white antialiased">{children}</body>
    </html>
  );
}
