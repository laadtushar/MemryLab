"use client";

import { motion, useScroll, useTransform } from "framer-motion";
import { useRef } from "react";
import {
  Brain,
  Search,
  Shield,
  Upload,
  Zap,
  GitBranch,
  BarChart3,
  MessageCircle,
  Sparkles,
  ChevronRight,
  Github,
} from "lucide-react";

const fadeUp = {
  hidden: { opacity: 0, y: 40 },
  visible: {
    opacity: 1,
    y: 0,
    transition: { duration: 0.6, ease: "easeOut" },
  },
};

const stagger = {
  visible: { transition: { staggerChildren: 0.1 } },
};

export default function Home() {
  const heroRef = useRef<HTMLDivElement>(null);
  const { scrollYProgress } = useScroll({
    target: heroRef,
    offset: ["start start", "end start"],
  });
  const heroOpacity = useTransform(scrollYProgress, [0, 1], [1, 0]);
  const heroY = useTransform(scrollYProgress, [0, 1], [0, -100]);

  return (
    <main className="relative">
      {/* Nav */}
      <nav className="fixed top-0 left-0 right-0 z-50 glass">
        <div className="max-w-6xl mx-auto px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-violet-500 to-blue-500 flex items-center justify-center text-white font-bold text-sm">
              M
            </div>
            <span className="text-lg font-bold">MemryLab</span>
          </div>
          <div className="flex items-center gap-6 text-sm text-zinc-400">
            <a href="#features" className="hover:text-white transition">
              Features
            </a>
            <a href="#sources" className="hover:text-white transition">
              Sources
            </a>
            <a href="/docs" className="hover:text-white transition">
              Docs
            </a>
            <a
              href="https://github.com/laadtushar/MemPalace"
              target="_blank"
              className="flex items-center gap-1 hover:text-white transition"
            >
              <Github size={16} /> GitHub
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

      {/* Hero */}
      <section
        ref={heroRef}
        className="relative min-h-screen flex items-center justify-center overflow-hidden"
      >
        {/* Animated gradient background */}
        <div className="absolute inset-0">
          <div className="absolute top-1/4 left-1/4 w-96 h-96 bg-violet-600/20 rounded-full blur-[128px] animate-pulse" />
          <div
            className="absolute bottom-1/4 right-1/4 w-96 h-96 bg-blue-600/20 rounded-full blur-[128px] animate-pulse"
            style={{ animationDelay: "1s" }}
          />
          <div
            className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-64 h-64 bg-cyan-500/10 rounded-full blur-[96px] animate-pulse"
            style={{ animationDelay: "2s" }}
          />
        </div>

        <motion.div
          style={{ opacity: heroOpacity, y: heroY }}
          className="relative z-10 text-center max-w-4xl mx-auto px-6"
        >
          <motion.div
            initial={{ opacity: 0, scale: 0.8 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ duration: 0.5 }}
          >
            <div className="inline-flex items-center gap-2 rounded-full glass px-4 py-2 text-sm text-zinc-300 mb-8">
              <Sparkles size={14} className="text-violet-400" />
              Open Source &amp; Privacy-First
            </div>
          </motion.div>

          <motion.h1
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2, duration: 0.7 }}
            className="text-6xl md:text-8xl font-bold tracking-tight leading-[0.9]"
          >
            Your Memory,
            <br />
            <span className="gradient-text">Visualized.</span>
          </motion.h1>

          <motion.p
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.4, duration: 0.6 }}
            className="text-xl text-zinc-400 mt-6 max-w-2xl mx-auto leading-relaxed"
          >
            MemryLab turns your digital footprint — journals, chats, social
            media, notes — into a searchable, visual timeline of how your
            thinking evolved. All on your device.
          </motion.p>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.6, duration: 0.5 }}
            className="flex items-center justify-center gap-4 mt-10"
          >
            <a
              href="https://github.com/laadtushar/MemPalace/releases"
              className="group flex items-center gap-2 px-8 py-4 rounded-full bg-violet-600 text-white font-semibold hover:bg-violet-500 transition glow"
            >
              Download Free{" "}
              <ChevronRight
                size={18}
                className="group-hover:translate-x-1 transition-transform"
              />
            </a>
            <a
              href="https://github.com/laadtushar/MemPalace"
              target="_blank"
              className="flex items-center gap-2 px-8 py-4 rounded-full glass text-zinc-300 font-medium hover:text-white transition"
            >
              <Github size={18} /> Star on GitHub
            </a>
          </motion.div>

          {/* Stats */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.8, duration: 0.6 }}
            className="flex items-center justify-center gap-8 mt-16 text-sm text-zinc-500"
          >
            <span>
              <strong className="text-white">30+</strong> import sources
            </span>
            <span className="w-px h-4 bg-zinc-700" />
            <span>
              <strong className="text-white">9</strong> AI providers
            </span>
            <span className="w-px h-4 bg-zinc-700" />
            <span>
              <strong className="text-white">4.7MB</strong> installer
            </span>
            <span className="w-px h-4 bg-zinc-700" />
            <span>
              <strong className="text-white">100%</strong> local
            </span>
          </motion.div>
        </motion.div>
      </section>

      {/* Features */}
      <section id="features" className="py-32 px-6">
        <div className="max-w-6xl mx-auto">
          <motion.div
            variants={fadeUp}
            initial="hidden"
            whileInView="visible"
            viewport={{ once: true }}
            className="text-center mb-16"
          >
            <h2 className="text-4xl md:text-5xl font-bold">
              Everything you need to
              <br />
              <span className="gradient-text">understand yourself.</span>
            </h2>
            <p className="text-zinc-400 mt-4 max-w-xl mx-auto">
              A complete toolkit for personal knowledge management, powered by
              local AI.
            </p>
          </motion.div>

          <motion.div
            variants={stagger}
            initial="hidden"
            whileInView="visible"
            viewport={{ once: true }}
            className="grid grid-cols-1 md:grid-cols-3 gap-6"
          >
            {[
              {
                icon: Upload,
                title: "30+ Import Sources",
                desc: "Google Takeout, WhatsApp, Twitter, Reddit, Obsidian, Notion, and more. Auto-detect any export format.",
              },
              {
                icon: Brain,
                title: "8-Stage Analysis",
                desc: "Themes, sentiment, beliefs, entities, insights, contradictions, and narrative generation.",
              },
              {
                icon: Search,
                title: "Hybrid Search + RAG",
                desc: "Keyword, semantic, and hybrid search. Ask questions and get answers grounded in your own writing.",
              },
              {
                icon: BarChart3,
                title: "Zoomable Timeline",
                desc: "Zoom from decades to days. See how your writing volume and themes evolved over time.",
              },
              {
                icon: GitBranch,
                title: "Knowledge Graph",
                desc: "Interactive force-directed graph showing people, places, concepts, and their relationships.",
              },
              {
                icon: Shield,
                title: "Privacy-First",
                desc: "All data stays on your device. SQLite database, OS keychain for secrets. Zero telemetry.",
              },
              {
                icon: Zap,
                title: "8 Free AI Providers",
                desc: "Gemini, Groq, OpenRouter, Cerebras, Mistral, SambaNova, Cohere — all free tiers supported.",
              },
              {
                icon: MessageCircle,
                title: "RAG Chat with History",
                desc: "Ask your memory like ChatGPT. Conversations saved, sources cited, context preserved.",
              },
              {
                icon: Sparkles,
                title: "4.7MB Installer",
                desc: "Tauri 2.0 + Rust. No Electron bloat. Native performance, tiny binary, cross-platform.",
              },
            ].map((f, i) => (
              <motion.div
                key={i}
                variants={fadeUp}
                className="glass rounded-2xl p-6 hover:border-violet-500/30 transition group"
              >
                <f.icon
                  size={24}
                  className="text-violet-400 mb-4 group-hover:scale-110 transition-transform"
                />
                <h3 className="text-lg font-semibold mb-2">{f.title}</h3>
                <p className="text-sm text-zinc-400 leading-relaxed">
                  {f.desc}
                </p>
              </motion.div>
            ))}
          </motion.div>
        </div>
      </section>

      {/* Import Sources */}
      <section id="sources" className="py-32 px-6 bg-zinc-950/50">
        <div className="max-w-6xl mx-auto">
          <motion.div
            variants={fadeUp}
            initial="hidden"
            whileInView="visible"
            viewport={{ once: true }}
            className="text-center mb-16"
          >
            <h2 className="text-4xl md:text-5xl font-bold">
              Import from <span className="gradient-text">anywhere.</span>
            </h2>
            <p className="text-zinc-400 mt-4">
              30+ platforms supported. Drop a ZIP, folder, or file — we figure
              out the rest.
            </p>
          </motion.div>

          <motion.div
            variants={stagger}
            initial="hidden"
            whileInView="visible"
            viewport={{ once: true }}
            className="flex flex-wrap justify-center gap-3"
          >
            {[
              "Google Takeout",
              "Facebook",
              "Instagram",
              "Twitter/X",
              "Reddit",
              "WhatsApp",
              "Telegram",
              "Discord",
              "Slack",
              "LinkedIn",
              "Obsidian",
              "Notion",
              "Evernote",
              "Day One",
              "Markdown",
              "Spotify",
              "YouTube",
              "Netflix",
              "TikTok",
              "Snapchat",
              "Bluesky",
              "Mastodon",
              "Substack",
              "Medium",
              "Tumblr",
              "Pinterest",
              "Apple",
              "Amazon",
              "Microsoft",
              "Signal",
            ].map((name, i) => (
              <motion.span
                key={i}
                variants={fadeUp}
                className="px-4 py-2 rounded-full glass text-sm text-zinc-300 hover:text-white hover:border-violet-500/30 transition cursor-default"
              >
                {name}
              </motion.span>
            ))}
          </motion.div>
        </div>
      </section>

      {/* CTA */}
      <section className="py-32 px-6 text-center">
        <motion.div
          variants={fadeUp}
          initial="hidden"
          whileInView="visible"
          viewport={{ once: true }}
        >
          <h2 className="text-4xl md:text-5xl font-bold mb-6">
            Ready to explore
            <br />
            <span className="gradient-text">your memory?</span>
          </h2>
          <p className="text-zinc-400 mb-10 max-w-lg mx-auto">
            Free, open source, and privacy-first. Your data never leaves your
            device.
          </p>
          <div className="flex items-center justify-center gap-4">
            <a
              href="https://github.com/laadtushar/MemPalace/releases"
              className="group flex items-center gap-2 px-8 py-4 rounded-full bg-violet-600 text-white font-semibold hover:bg-violet-500 transition glow"
            >
              Download for Free{" "}
              <ChevronRight
                size={18}
                className="group-hover:translate-x-1 transition-transform"
              />
            </a>
            <a
              href="https://github.com/laadtushar/MemPalace"
              target="_blank"
              className="flex items-center gap-2 px-8 py-4 rounded-full glass text-zinc-300 font-medium hover:text-white transition"
            >
              <Github size={18} /> View Source
            </a>
          </div>
        </motion.div>
      </section>

      {/* Footer */}
      <footer className="border-t border-zinc-800 py-12 px-6">
        <div className="max-w-6xl mx-auto flex flex-col md:flex-row items-center justify-between gap-6">
          <div className="flex items-center gap-2">
            <div className="w-6 h-6 rounded bg-gradient-to-br from-violet-500 to-blue-500 flex items-center justify-center text-white font-bold text-xs">
              M
            </div>
            <span className="font-semibold">MemryLab</span>
            <span className="text-zinc-500 text-sm">
              — Your memory, visualized.
            </span>
          </div>
          <div className="flex items-center gap-6 text-sm text-zinc-500">
            <a
              href="https://github.com/laadtushar/MemPalace"
              className="hover:text-white transition flex items-center gap-1"
            >
              <Github size={14} /> GitHub
            </a>
            <a href="/docs" className="hover:text-white transition">
              Docs
            </a>
            <a
              href="https://www.linkedin.com/in/tusharlaad2002/"
              className="hover:text-white transition"
            >
              Creator
            </a>
            <a
              href="https://github.com/laadtushar/MemPalace/blob/master/CONTRIBUTING.md"
              className="hover:text-white transition"
            >
              Contribute
            </a>
          </div>
        </div>
      </footer>
    </main>
  );
}
