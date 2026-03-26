"use client";

import { motion, useScroll, useTransform } from "framer-motion";
import { useRef, useState } from "react";
import {
  Brain, Search, Shield, Upload, Zap, GitBranch,
  BarChart3, MessageCircle, Sparkles, ChevronRight,
  Github, ArrowRight, Lock, Cpu, Globe, Download,
} from "lucide-react";
import AppPreview from "@/components/AppPreview";

const fadeUp = {
  hidden: { opacity: 0, y: 30 },
  visible: { opacity: 1, y: 0, transition: { duration: 0.5, ease: "easeOut" } },
};
const fadeIn = {
  hidden: { opacity: 0 },
  visible: { opacity: 1, transition: { duration: 0.6 } },
};
const stagger = {
  visible: { transition: { staggerChildren: 0.08 } },
};
const scaleIn = {
  hidden: { opacity: 0, scale: 0.9 },
  visible: { opacity: 1, scale: 1, transition: { duration: 0.5, ease: "easeOut" } },
};

// Inline SVG logo — simplified for small, detailed for large
function Logo({ size = 32 }: { size?: number }) {
  const id = `logo-${size}`;
  return (
    <svg width={size} height={size} viewBox="0 0 100 100" fill="none" xmlns="http://www.w3.org/2000/svg">
      <defs>
        <linearGradient id={id} x1="0" y1="0" x2="100" y2="100" gradientUnits="userSpaceOnUse">
          <stop offset="0%" stopColor="#6d28d9"/><stop offset="50%" stopColor="#4f46e5"/><stop offset="100%" stopColor="#0ea5e9"/>
        </linearGradient>
      </defs>
      <rect width="100" height="100" rx="22" fill={`url(#${id})`}/>
      {/* Connections */}
      <line x1="50" y1="45" x2="30" y2="30" stroke="white" strokeWidth="2.5" opacity="0.6"/>
      <line x1="50" y1="45" x2="70" y2="30" stroke="white" strokeWidth="2.5" opacity="0.6"/>
      <line x1="50" y1="55" x2="28" y2="65" stroke="white" strokeWidth="2" opacity="0.5"/>
      <line x1="50" y1="55" x2="72" y2="65" stroke="white" strokeWidth="2" opacity="0.5"/>
      <line x1="50" y1="58" x2="40" y2="78" stroke="white" strokeWidth="2" opacity="0.45"/>
      <line x1="50" y1="58" x2="60" y2="78" stroke="white" strokeWidth="2" opacity="0.45"/>
      <line x1="50" y1="42" x2="50" y2="22" stroke="white" strokeWidth="2.5" opacity="0.55"/>
      {/* Nodes */}
      <circle cx="50" cy="50" r="9" fill="white"/>
      <circle cx="50" cy="22" r="5" fill="white" opacity="0.9"/>
      <circle cx="30" cy="30" r="4.5" fill="white" opacity="0.85"/>
      <circle cx="70" cy="30" r="4.5" fill="white" opacity="0.85"/>
      <circle cx="28" cy="65" r="4" fill="white" opacity="0.75"/>
      <circle cx="72" cy="65" r="4" fill="white" opacity="0.75"/>
      <circle cx="40" cy="78" r="3.5" fill="white" opacity="0.7"/>
      <circle cx="60" cy="78" r="3.5" fill="white" opacity="0.7"/>
      {/* Pulse ring */}
      <circle cx="50" cy="50" r="13" stroke="white" strokeWidth="1" fill="none" opacity="0.2"/>
      {/* M */}
      <text x="50" y="54" textAnchor="middle" fontFamily="system-ui,sans-serif" fontWeight="900" fontSize="10" fill="#4f46e5">M</text>
    </svg>
  );
}

const PLATFORMS = [
  "Google Takeout", "Facebook", "Instagram", "Twitter/X", "Reddit",
  "WhatsApp", "Telegram", "Discord", "Slack", "LinkedIn",
  "Obsidian", "Notion", "Evernote", "Day One", "Markdown",
  "Spotify", "YouTube", "Netflix", "TikTok", "Snapchat",
  "Bluesky", "Mastodon", "Substack", "Medium", "Tumblr",
  "Pinterest", "Apple", "Amazon", "Microsoft", "Signal",
];

const FEATURES = [
  { icon: Upload, title: "30+ Import Sources", desc: "Google, WhatsApp, Twitter, Reddit, Obsidian, Notion — auto-detect any export format.", color: "from-violet-500/20 to-violet-500/5" },
  { icon: Brain, title: "8-Stage AI Analysis", desc: "Themes, sentiment, beliefs, entities, insights, contradictions, narratives — all extracted automatically.", color: "from-blue-500/20 to-blue-500/5" },
  { icon: Search, title: "Hybrid Search + RAG", desc: "Keyword, semantic, hybrid search. Ask questions grounded in your own writing with source citations.", color: "from-cyan-500/20 to-cyan-500/5" },
  { icon: BarChart3, title: "Zoomable Timeline", desc: "Zoom from decades to individual days. Watch how your writing volume and themes evolved.", color: "from-emerald-500/20 to-emerald-500/5" },
  { icon: GitBranch, title: "Knowledge Graph", desc: "Interactive force-directed graph of people, places, concepts, and how they connect in your life.", color: "from-amber-500/20 to-amber-500/5" },
  { icon: Shield, title: "Privacy-First", desc: "All data stays on your device. SQLite database, OS keychain for secrets. Zero telemetry, ever.", color: "from-rose-500/20 to-rose-500/5" },
];

const BOTTOM_FEATURES = [
  { icon: Zap, label: "8 Free AI Providers", desc: "Gemini, Groq, OpenRouter, Cerebras, Mistral, SambaNova, Cohere" },
  { icon: MessageCircle, label: "Chat with Your Memory", desc: "Ask questions like ChatGPT — answers come from your own documents" },
  { icon: Lock, label: "OS Keychain Security", desc: "API keys in Windows Credential Manager / macOS Keychain" },
  { icon: Cpu, label: "4.7MB Installer", desc: "Tauri 2.0 + Rust. No Electron bloat. Native performance." },
  { icon: Globe, label: "Cross-Platform", desc: "Windows, macOS, Linux. One codebase, three platforms." },
  { icon: Sparkles, label: "Open Source", desc: "MIT licensed. Contribute, fork, or self-host." },
];

export default function Home() {
  const heroRef = useRef<HTMLDivElement>(null);
  const { scrollYProgress } = useScroll({ target: heroRef, offset: ["start start", "end start"] });
  const heroOpacity = useTransform(scrollYProgress, [0, 1], [1, 0]);
  const heroScale = useTransform(scrollYProgress, [0, 1], [1, 0.95]);
  const [hoveredFeature, setHoveredFeature] = useState<number | null>(null);

  return (
    <main className="relative overflow-x-hidden">
      {/* ── Nav ── */}
      <motion.nav
        initial={{ y: -20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ delay: 0.1, duration: 0.4 }}
        className="fixed top-0 left-0 right-0 z-50"
      >
        <div className="max-w-6xl mx-auto px-6 py-3 mt-3">
          <div className="glass rounded-2xl px-6 py-3 flex items-center justify-between">
            <a href="/" className="flex items-center gap-2.5">
              <Logo size={28} />
              <span className="text-lg font-bold tracking-tight">MemryLab</span>
            </a>
            <div className="hidden md:flex items-center gap-6 text-sm text-zinc-400">
              <a href="#features" className="hover:text-white transition">Features</a>
              <a href="#preview" className="hover:text-white transition">Preview</a>
              <a href="#sources" className="hover:text-white transition">Sources</a>
              <a href="/docs" className="hover:text-white transition">Docs</a>
              <a href="https://github.com/laadtushar/MemryLab" target="_blank" rel="noopener noreferrer" className="flex items-center gap-1.5 hover:text-white transition">
                <Github size={15} /> GitHub
              </a>
            </div>
            <a href="https://github.com/laadtushar/MemryLab/releases" className="flex items-center gap-2 px-5 py-2 rounded-xl bg-gradient-to-r from-violet-600 to-indigo-600 text-white text-sm font-semibold hover:from-violet-500 hover:to-indigo-500 transition-all shadow-lg shadow-violet-600/25">
              <Download size={14} /> Download
            </a>
          </div>
        </div>
      </motion.nav>

      {/* ── Hero ── */}
      <section ref={heroRef} className="relative min-h-screen flex items-center justify-center overflow-hidden pt-24">
        {/* Animated background */}
        <div className="absolute inset-0 overflow-hidden">
          <div className="absolute top-[20%] left-[15%] w-[500px] h-[500px] bg-violet-600/15 rounded-full blur-[160px] animate-pulse" />
          <div className="absolute bottom-[20%] right-[15%] w-[400px] h-[400px] bg-indigo-600/15 rounded-full blur-[140px] animate-pulse" style={{ animationDelay: "1.5s" }} />
          <div className="absolute top-[40%] left-[50%] -translate-x-1/2 w-[300px] h-[300px] bg-cyan-500/8 rounded-full blur-[120px] animate-pulse" style={{ animationDelay: "3s" }} />
          {/* Grid pattern */}
          <div className="absolute inset-0 opacity-[0.03]" style={{ backgroundImage: "linear-gradient(rgba(255,255,255,0.1) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.1) 1px, transparent 1px)", backgroundSize: "60px 60px" }} />
        </div>

        <motion.div style={{ opacity: heroOpacity, scale: heroScale }} className="relative z-10 text-center max-w-5xl mx-auto px-6">
          {/* Badge */}
          <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.5 }}>
            <a href="https://github.com/laadtushar/MemryLab" target="_blank" rel="noopener noreferrer"
              className="inline-flex items-center gap-2 rounded-full glass px-4 py-2 text-sm text-zinc-300 mb-8 hover:border-violet-500/30 transition group">
              <span className="flex h-2 w-2 rounded-full bg-emerald-400 animate-pulse" />
              Open Source & Privacy-First
              <ArrowRight size={13} className="text-zinc-500 group-hover:text-white group-hover:translate-x-0.5 transition-all" />
            </a>
          </motion.div>

          {/* Logo + Heading */}
          <motion.div initial={{ opacity: 0, scale: 0.8 }} animate={{ opacity: 1, scale: 1 }} transition={{ delay: 0.15, duration: 0.6 }}
            className="flex justify-center">
            <Logo size={80} />
          </motion.div>

          <motion.h1 initial={{ opacity: 0, y: 25 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.25, duration: 0.6 }}
            className="text-5xl sm:text-7xl md:text-8xl font-bold tracking-tight leading-[0.95] mt-6">
            Your Memory,<br />
            <span className="gradient-text">Visualized.</span>
          </motion.h1>

          <motion.p initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.4, duration: 0.5 }}
            className="text-lg md:text-xl text-zinc-400 mt-6 max-w-2xl mx-auto leading-relaxed">
            Turn your digital footprint — journals, chats, social media, notes — into a searchable timeline of how your thinking evolved.{" "}
            <span className="text-zinc-300">All on your device.</span>
          </motion.p>

          {/* CTAs */}
          <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.55, duration: 0.5 }}
            className="flex flex-col sm:flex-row items-center justify-center gap-4 mt-10">
            <a href="https://github.com/laadtushar/MemryLab/releases"
              className="group flex items-center gap-2.5 px-8 py-4 rounded-2xl bg-gradient-to-r from-violet-600 to-indigo-600 text-white font-semibold hover:from-violet-500 hover:to-indigo-500 transition-all shadow-xl shadow-violet-600/30 hover:shadow-violet-500/40 hover:-translate-y-0.5">
              <Download size={18} />
              Download Free
              <ChevronRight size={16} className="group-hover:translate-x-1 transition-transform" />
            </a>
            <a href="https://github.com/laadtushar/MemryLab" target="_blank" rel="noopener noreferrer"
              className="flex items-center gap-2.5 px-8 py-4 rounded-2xl glass text-zinc-300 font-medium hover:text-white hover:border-violet-500/20 transition-all hover:-translate-y-0.5">
              <Github size={18} /> Star on GitHub
            </a>
          </motion.div>

          {/* Stats */}
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: 0.7, duration: 0.5 }}
            className="flex flex-wrap items-center justify-center gap-6 md:gap-10 mt-16">
            {[
              { value: "30+", label: "Import Sources" },
              { value: "9", label: "AI Providers" },
              { value: "4.7MB", label: "Installer Size" },
              { value: "100%", label: "Local & Private" },
            ].map((s, i) => (
              <div key={i} className="text-center">
                <div className="text-2xl font-bold gradient-text">{s.value}</div>
                <div className="text-xs text-zinc-500 mt-0.5">{s.label}</div>
              </div>
            ))}
          </motion.div>
        </motion.div>

        {/* Scroll indicator */}
        <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: 1.2 }}
          className="absolute bottom-8 left-1/2 -translate-x-1/2 flex flex-col items-center gap-2 text-zinc-600">
          <span className="text-xs">Scroll to explore</span>
          <motion.div animate={{ y: [0, 6, 0] }} transition={{ repeat: Infinity, duration: 1.5 }}
            className="w-5 h-8 rounded-full border border-zinc-700 flex items-start justify-center pt-1.5">
            <div className="w-1 h-1.5 rounded-full bg-zinc-500" />
          </motion.div>
        </motion.div>
      </section>

      {/* ── Features ── */}
      <section id="features" className="py-32 px-6">
        <div className="max-w-6xl mx-auto">
          <motion.div variants={fadeUp} initial="hidden" whileInView="visible" viewport={{ once: true }} className="text-center mb-20">
            <span className="text-sm font-medium text-violet-400 tracking-wider uppercase">Features</span>
            <h2 className="text-4xl md:text-5xl font-bold mt-3">
              Everything you need to<br />
              <span className="gradient-text">understand yourself.</span>
            </h2>
          </motion.div>

          <motion.div variants={stagger} initial="hidden" whileInView="visible" viewport={{ once: true }} className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
            {FEATURES.map((f, i) => (
              <motion.div key={i} variants={scaleIn}
                onMouseEnter={() => setHoveredFeature(i)}
                onMouseLeave={() => setHoveredFeature(null)}
                className={`relative rounded-2xl p-6 border border-white/5 bg-gradient-to-br ${f.color} backdrop-blur-sm transition-all duration-300 cursor-default ${hoveredFeature === i ? "border-violet-500/30 scale-[1.02] shadow-xl shadow-violet-600/10" : "hover:border-white/10"}`}
              >
                <div className="flex items-center gap-3 mb-4">
                  <div className="w-10 h-10 rounded-xl bg-white/10 flex items-center justify-center">
                    <f.icon size={20} className="text-white" />
                  </div>
                  <h3 className="text-lg font-semibold">{f.title}</h3>
                </div>
                <p className="text-sm text-zinc-400 leading-relaxed">{f.desc}</p>
              </motion.div>
            ))}
          </motion.div>

          {/* Bottom feature strip */}
          <motion.div variants={stagger} initial="hidden" whileInView="visible" viewport={{ once: true }}
            className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4 mt-12">
            {BOTTOM_FEATURES.map((f, i) => (
              <motion.div key={i} variants={fadeUp}
                className="glass rounded-xl p-4 text-center hover:border-violet-500/20 transition">
                <f.icon size={18} className="mx-auto text-violet-400 mb-2" />
                <div className="text-xs font-semibold mb-1">{f.label}</div>
                <div className="text-[10px] text-zinc-500 leading-tight">{f.desc}</div>
              </motion.div>
            ))}
          </motion.div>
        </div>
      </section>

      {/* ── Sneak Peek ── */}
      <section id="preview" className="py-32 px-6 relative">
        <div className="absolute inset-0 bg-gradient-to-b from-transparent via-violet-600/5 to-transparent" />
        <div className="max-w-5xl mx-auto relative">
          <motion.div variants={fadeUp} initial="hidden" whileInView="visible" viewport={{ once: true }} className="text-center mb-14">
            <span className="text-sm font-medium text-violet-400 tracking-wider uppercase">Preview</span>
            <h2 className="text-4xl md:text-5xl font-bold mt-3">See it in <span className="gradient-text">action.</span></h2>
            <p className="text-zinc-400 mt-3">A glimpse of what MemryLab looks like on your desktop.</p>
          </motion.div>
          <motion.div initial={{ opacity: 0, y: 50 }} whileInView={{ opacity: 1, y: 0 }} viewport={{ once: true }} transition={{ duration: 0.8, ease: "easeOut" }}>
            <div className="glow rounded-2xl">
              <AppPreview />
            </div>
          </motion.div>
        </div>
      </section>

      {/* ── Import Sources ── */}
      <section id="sources" className="py-32 px-6">
        <div className="max-w-6xl mx-auto">
          <motion.div variants={fadeUp} initial="hidden" whileInView="visible" viewport={{ once: true }} className="text-center mb-14">
            <span className="text-sm font-medium text-violet-400 tracking-wider uppercase">Compatibility</span>
            <h2 className="text-4xl md:text-5xl font-bold mt-3">Import from <span className="gradient-text">anywhere.</span></h2>
            <p className="text-zinc-400 mt-3 max-w-lg mx-auto">30+ platforms supported. Drop a ZIP, folder, or file — we auto-detect the format and extract everything.</p>
          </motion.div>

          <motion.div variants={stagger} initial="hidden" whileInView="visible" viewport={{ once: true }}
            className="flex flex-wrap justify-center gap-2.5">
            {PLATFORMS.map((name, i) => (
              <motion.span key={i} variants={fadeIn}
                className="px-4 py-2 rounded-full text-sm border border-white/5 bg-white/[0.03] text-zinc-400 hover:text-white hover:border-violet-500/30 hover:bg-violet-500/10 transition-all duration-200 cursor-default">
                {name}
              </motion.span>
            ))}
          </motion.div>
        </div>
      </section>

      {/* ── CTA ── */}
      <section className="py-32 px-6 relative">
        <div className="absolute inset-0 overflow-hidden">
          <div className="absolute bottom-0 left-1/2 -translate-x-1/2 w-[600px] h-[300px] bg-violet-600/10 rounded-full blur-[160px]" />
        </div>
        <motion.div variants={fadeUp} initial="hidden" whileInView="visible" viewport={{ once: true }} className="relative text-center max-w-2xl mx-auto">
          <Logo size={48} />
          <h2 className="text-4xl md:text-5xl font-bold mt-6 mb-4">
            Ready to explore<br /><span className="gradient-text">your memory?</span>
          </h2>
          <p className="text-zinc-400 mb-10">Free, open source, privacy-first. Your data never leaves your device.</p>
          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <a href="https://github.com/laadtushar/MemryLab/releases"
              className="group flex items-center gap-2.5 px-8 py-4 rounded-2xl bg-gradient-to-r from-violet-600 to-indigo-600 text-white font-semibold hover:from-violet-500 hover:to-indigo-500 transition-all shadow-xl shadow-violet-600/30 hover:-translate-y-0.5">
              <Download size={18} /> Download for Free
              <ChevronRight size={16} className="group-hover:translate-x-1 transition-transform" />
            </a>
            <a href="https://github.com/laadtushar/MemryLab" target="_blank" rel="noopener noreferrer"
              className="flex items-center gap-2.5 px-8 py-4 rounded-2xl glass text-zinc-300 font-medium hover:text-white transition-all hover:-translate-y-0.5">
              <Github size={18} /> View Source
            </a>
          </div>
        </motion.div>
      </section>

      {/* ── Footer ── */}
      <footer className="border-t border-zinc-800/50 py-16 px-6">
        <div className="max-w-6xl mx-auto">
          <div className="flex flex-col md:flex-row items-start justify-between gap-10">
            <div>
              <div className="flex items-center gap-2.5 mb-3">
                <Logo size={24} />
                <span className="font-bold text-lg">MemryLab</span>
              </div>
              <p className="text-sm text-zinc-500 max-w-xs">A searchable, visual timeline of how your thinking evolved. Privacy-first, open source.</p>
            </div>
            <div className="grid grid-cols-2 gap-x-16 gap-y-3 text-sm">
              <a href="/docs" className="text-zinc-400 hover:text-white transition">Documentation</a>
              <a href="https://github.com/laadtushar/MemryLab" target="_blank" rel="noopener noreferrer" className="text-zinc-400 hover:text-white transition flex items-center gap-1.5"><Github size={13} /> GitHub</a>
              <a href="/docs/getting-started" className="text-zinc-400 hover:text-white transition">Getting Started</a>
              <a href="https://github.com/laadtushar/MemryLab/blob/master/CONTRIBUTING.md" target="_blank" rel="noopener noreferrer" className="text-zinc-400 hover:text-white transition">Contribute</a>
              <a href="/docs/ai-providers" className="text-zinc-400 hover:text-white transition">AI Providers</a>
              <a href="https://www.linkedin.com/in/tusharlaad2002/" target="_blank" rel="noopener noreferrer" className="text-zinc-400 hover:text-white transition">Creator</a>
            </div>
          </div>
          <div className="mt-12 pt-6 border-t border-zinc-800/50 flex flex-col sm:flex-row items-center justify-between gap-4 text-xs text-zinc-600">
            <span>&copy; {new Date().getFullYear()} MemryLab. Open source under MIT License.</span>
            <span>Built with Rust, React, Tauri, and D3.js</span>
          </div>
        </div>
      </footer>
    </main>
  );
}
