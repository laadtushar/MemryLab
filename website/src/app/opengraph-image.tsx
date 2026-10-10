import { ImageResponse } from "next/og";
import { OG_IMAGE_ALT, SITE_NAME, TAGLINE } from "@/lib/site";

export const alt = OG_IMAGE_ALT;
export const size = { width: 1200, height: 630 };
export const contentType = "image/png";

export default function OpenGraphImage() {
  return new ImageResponse(
    (
      <div
        style={{
          width: "100%",
          height: "100%",
          display: "flex",
          flexDirection: "column",
          justifyContent: "center",
          padding: "0 96px",
          background: "linear-gradient(135deg, #0b0614 0%, #1e1b4b 60%, #0c4a6e 100%)",
          color: "#fafafa",
          fontFamily: "sans-serif",
        }}
      >
        <div style={{ display: "flex", alignItems: "center" }}>
          <svg width="112" height="112" viewBox="0 0 100 100">
            <defs>
              <linearGradient id="g" x1="0" y1="0" x2="100" y2="100" gradientUnits="userSpaceOnUse">
                <stop offset="0%" stopColor="#6d28d9" />
                <stop offset="50%" stopColor="#4f46e5" />
                <stop offset="100%" stopColor="#0ea5e9" />
              </linearGradient>
            </defs>
            <rect width="100" height="100" rx="22" fill="url(#g)" />
            <line x1="50" y1="50" x2="30" y2="30" stroke="white" strokeWidth="2.5" opacity="0.6" />
            <line x1="50" y1="50" x2="70" y2="30" stroke="white" strokeWidth="2.5" opacity="0.6" />
            <line x1="50" y1="50" x2="28" y2="65" stroke="white" strokeWidth="2" opacity="0.5" />
            <line x1="50" y1="50" x2="72" y2="65" stroke="white" strokeWidth="2" opacity="0.5" />
            <line x1="50" y1="50" x2="50" y2="22" stroke="white" strokeWidth="2.5" opacity="0.55" />
            <circle cx="50" cy="50" r="9" fill="white" />
            <circle cx="50" cy="22" r="5" fill="white" />
            <circle cx="30" cy="30" r="4.5" fill="white" />
            <circle cx="70" cy="30" r="4.5" fill="white" />
            <circle cx="28" cy="65" r="4" fill="white" />
            <circle cx="72" cy="65" r="4" fill="white" />
          </svg>
          <div style={{ fontSize: 84, fontWeight: 800, marginLeft: 32, letterSpacing: -2 }}>{SITE_NAME}</div>
        </div>
        <div style={{ fontSize: 68, fontWeight: 700, marginTop: 56, lineHeight: 1.1, color: "#a78bfa" }}>{TAGLINE}</div>
        <div style={{ fontSize: 34, marginTop: 28, color: "#a1a1aa" }}>
          Local AI memory. Your journals, chats and exports as a searchable timeline.
        </div>
        <div style={{ fontSize: 28, marginTop: 44, color: "#71717a" }}>www.memrylab.com  ·  Free and open source</div>
      </div>
    ),
    { ...size },
  );
}
