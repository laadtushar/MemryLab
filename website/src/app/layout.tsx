import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "MemryLab — Your Memory, Visualized",
  description:
    "A privacy-first desktop app that turns your digital footprint into a searchable, visual timeline of personal evolution.",
  openGraph: {
    title: "MemryLab",
    description:
      "Your memory, visualized. Privacy-first personal knowledge management.",
    url: "https://memrylab.com",
    siteName: "MemryLab",
    type: "website",
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark">
      <body className="bg-black text-white antialiased">{children}</body>
    </html>
  );
}
