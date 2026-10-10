import { WINDOWS_INSTALLER_SIZE } from "@/lib/site";

export interface Article {
  slug: string;
  title: string;
  /** Shorter title for <title> and og:title (about 50-60 characters). */
  metaTitle: string;
  excerpt: string;
  /** Meta description (about 140-160 characters). */
  description: string;
  /** Publication date (ISO, YYYY-MM-DD). */
  date: string;
  /** Date of the last edit (ISO), set when a post is revised after publication. */
  dateModified?: string;
  author: string;
  readTime: string;
  tags: string[];
  content: string;
}

export const articles: Article[] = [
  {
    slug: "why-i-built-memrylab",
    title: "Why I Built MemryLab: Tracking How My Beliefs Changed Over 3 Years",
    metaTitle: "Why I Built MemryLab: Tracking How My Beliefs Changed",
    excerpt: "I had 14,000 documents across Google, WhatsApp, journals, and Twitter. No tool could tell me how my thinking evolved. So I built one.",
    description: "I had 14,000 documents across Google, WhatsApp, journals and Twitter and no way to see how my thinking changed. So I built MemryLab, a private desktop app.",
    date: "2026-03-15",
    dateModified: "2026-10-10",
    author: "Tushar Laad",
    readTime: "8 min read",
    tags: ["launch", "personal"],
    content: `# Why I Built MemryLab

In late 2023, I downloaded my Google Takeout archive. 47 GB of emails, searches, location history, and YouTube watches spanning a decade. I also had three years of Day One journal entries, an Obsidian vault with 800+ notes, a Twitter archive, and WhatsApp exports from half a dozen group chats.

I had *all* the data. What I didn't have was any way to understand it.

## The Moment That Started Everything

I was reading through old journal entries when I found this from March 2022: *"I value stability and predictability in my career. Taking risks feels irresponsible."*

Then I found a Twitter thread I'd written in November 2023: *"The biggest growth moments came from embracing uncertainty. Playing it safe is the real risk."*

I'd completely reversed my position on risk — and I hadn't even noticed until I stumbled across both entries by accident. How many other belief changes were hiding in my data?

## What Exists Today (And Why It Fails)

I tried every personal knowledge tool I could find:

- **Notion/Obsidian**: Great for writing, terrible for cross-platform analysis
- **Google Takeout viewer**: Raw data dump, no intelligence
- **MyLifeBits-style apps**: Focus on storage, not insight
- **ChatGPT**: Can't access my local files, and I don't want to upload 47 GB of personal data to OpenAI

The fundamental problem: **every tool treats your documents as independent retrieval targets.** They answer "what did I write?" but never "how has my thinking changed?"

I call this the *meta-narrative problem* — and it's what MemryLab solves.

## How MemryLab Works

MemryLab is a desktop app (a ${WINDOWS_INSTALLER_SIZE} Windows installer) that:

1. **Imports from 30+ platforms** — Google Takeout, WhatsApp, Twitter, Obsidian, Reddit, Telegram, Day One, and more. Drop a ZIP or folder; it auto-detects the format.

2. **Extracts structured beliefs** — An 8-stage AI pipeline identifies themes, sentiments, beliefs, entities, insights, contradictions, and narratives from your writing.

3. **Detects contradictions** — The core innovation: embedding-based pre-filtering + LLM verification identifies when you've changed your mind. My career-risk reversal? MemryLab found it automatically.

4. **Generates evolution narratives** — Second-person prose ("Your writing shows...") that tells the story of how your thinking shifted, grounded in your own words.

5. **Local by default** — Ollama is the default provider, so your data stays on your device unless you choose a cloud model.

## What I Found In My Own Data

Running MemryLab on my 14,653 documents revealed:

- **47 unique themes** across 24 months, with career development appearing in 22 of them
- **127 belief-level facts** extracted from my writing
- **4 genuine contradictions** — including the career-risk reversal that started all of this
- A clear **evolution arc** from "optimize for stability" to "optimize for growth" between 2022-2023

The most surprising finding: my writing about *creative work* shifted dramatically — from treating it as a hobby to treating it as a core identity. I never consciously made that transition, but MemryLab showed me the exact month it happened.

## Open Source, Free, Local by Default

MemryLab is MIT-licensed. It supports 9 AI providers (8 with free tiers). The Windows installer is ${WINDOWS_INSTALLER_SIZE}. Local by default: your data stays on your device unless you choose a cloud model.

If you've ever wondered how your thinking has changed — or if you've changed in ways you haven't consciously recognized — give it a try.

[Download MemryLab](https://github.com/laadtushar/MemryLab/releases/latest) | [Read the Docs](https://www.memrylab.com/docs)`,
  },
  {
    slug: "meta-narrative-problem",
    title: "The Meta-Narrative Problem: Why Search Isn't Enough for Personal Data",
    metaTitle: "The Meta-Narrative Problem: Why Search Isn't Enough",
    excerpt: "Every personal data tool answers 'what did I write?' None answer 'how has my thinking changed?' That's the meta-narrative problem.",
    description: "Search answers what did I write, never how has my thinking changed. Why personal data tools miss the story and what it takes to surface it from your writing.",
    date: "2026-03-10",
    dateModified: "2026-10-10",
    author: "Tushar Laad",
    readTime: "6 min read",
    tags: ["research", "thinking"],
    content: `# The Meta-Narrative Problem

## What Search Gets Wrong

When you search your old emails, journal entries, or social media posts, you're asking: *"What did I write about X?"*

But the more interesting question is: *"How have my views on X changed over time?"*

No existing tool answers this. Not Google, not Notion, not Obsidian, not any digital memory system. They're all built around the same paradigm: **documents as independent retrieval targets.**

I call this gap the *meta-narrative problem*: the inability to understand the story your data tells about your personal evolution.

## Why It Matters

Consider these scenarios:

- You journaled about "valuing independence" in 2020. By 2024, you're writing about "the importance of community." Neither entry references the other. No search query would surface this shift.

- Your Reddit comments in 2021 were skeptical of remote work. Your LinkedIn posts in 2023 celebrate it. These live on different platforms with different exports. No tool connects them.

- Your Obsidian notes from 2022 express confidence in a career direction. Your WhatsApp messages from 2023 reveal uncertainty about the same topic. No system correlates beliefs across platforms.

## The Four Requirements

Solving the meta-narrative problem requires four capabilities that no existing tool combines:

1. **Multi-platform data fusion** — Ingest heterogeneous exports from dozens of platforms into a single temporal knowledge base
2. **Belief extraction** — Move beyond keyword matching to identify structured beliefs, values, and self-descriptions from unstructured text
3. **Temporal contradiction detection** — Compare beliefs across time to find genuine shifts, not just topic overlap
4. **Narrative synthesis** — Generate human-readable stories about how thinking evolved, grounded in original writing

## How LLMs Change the Game

Before LLMs, each of these steps required task-specific training. Extracting beliefs from text needed a custom NLP model. Generating coherent narratives needed another. The engineering cost was prohibitive for a personal tool.

Modern LLMs can do all four zero-shot. The challenge shifts from "can we do this?" to "can we do this efficiently and privately?"

MemryLab's answer: embedding-based pre-filtering reduces LLM calls from O(n^2) to O(k), where k <= 20. The entire pipeline runs in ~25 minutes on a consumer laptop with a local 8B model.

## The Bigger Picture

The meta-narrative problem isn't just a technical curiosity. It's about **self-knowledge** — the kind that only comes from seeing patterns you couldn't see in the moment.

When MemryLab surfaces a contradiction between your 2021 self and your 2024 self, it's not pointing out a flaw. It's showing you growth. And that's worth building for.

[Try MemryLab](https://github.com/laadtushar/MemryLab/releases/latest) | [Read the Research Paper](https://github.com/laadtushar/MemryLab/tree/master/paper)`,
  },
  {
    slug: "running-ai-locally-ollama-guide",
    title: "Running AI Locally: A Guide to Private LLM Analysis with Ollama",
    metaTitle: "Running AI Locally: Private LLM Analysis with Ollama",
    excerpt: "Step-by-step guide to setting up local AI inference with Ollama. Which models to use for 4GB, 8GB, 12GB, and 16GB+ VRAM.",
    description: "Set up local AI inference with Ollama for private analysis. Which models to run on 4GB, 8GB, 12GB and 16GB+ of VRAM, and why to keep embeddings local.",
    date: "2026-03-05",
    dateModified: "2026-10-10",
    author: "Tushar Laad",
    readTime: "5 min read",
    tags: ["tutorial", "ollama"],
    content: `# Running AI Locally with Ollama

## Why Local AI?

Cloud AI is convenient, but for personal data analysis, privacy matters. When you're extracting beliefs, tracking contradictions, and generating narratives from your most personal writing, you want that processing to happen on *your* machine.

Ollama makes this practical. It runs open-source LLMs locally with GPU acceleration — no internet required.

## Setup (2 Minutes)

1. Download Ollama from [ollama.com](https://ollama.com)
2. Install and run it (it starts a local server at localhost:11434)
3. Pull two models:

\`\`\`bash
# Embedding model (required for search, 274 MB)
ollama pull nomic-embed-text

# LLM model (pick by your VRAM)
ollama pull qwen2.5:14b-instruct-q4_K_M
\`\`\`

## Which Model For Your Hardware?

| VRAM | Model | Quality | Speed |
|------|-------|---------|-------|
| **4 GB** | llama3.2:3b | Good for basic extraction | ~40 tok/s |
| **8 GB** | llama3.1:8b | Solid all-around | ~35 tok/s |
| **12 GB** | qwen2.5:14b-instruct-q4_K_M | Best quality/speed balance | ~25 tok/s |
| **16 GB+** | qwen2.5:32b-instruct-q4_K_M | Near-cloud quality | ~15 tok/s |

**Recommendation:** If you have 12 GB VRAM (RTX 4070/5070 Ti), the 14B Qwen model is the sweet spot. It's significantly better than 8B models at structured JSON extraction and nuanced belief analysis.

## Embedding Model: Always Use nomic-embed-text

For search to work, you need an embedding model. \`nomic-embed-text\` is:
- Only 274 MB
- 768 dimensions (efficient)
- Fast enough for real-time search
- Runs on any hardware

## Hybrid Setup: Cloud LLM + Local Embeddings

The best of both worlds: use a free cloud LLM (Gemini Flash, Groq) for analysis + local Ollama for embeddings. This way:
- Analysis uses a powerful cloud model (free)
- Your search index stays fully private (local)
- No VRAM needed for the large LLM

In MemryLab: Settings > Embedding Provider > Select "Ollama (local, private)"

## Cost Comparison

| Setup | Monthly Cost | Privacy | Speed |
|-------|-------------|---------|-------|
| Full local (Ollama) | $0 + electricity | Full | 15-40 tok/s |
| Gemini Flash (cloud) | ~$0 (free tier) | Analysis only | 100+ tok/s |
| GPT-4o (cloud) | ~$5-20 | None | 60+ tok/s |

For MemryLab's typical usage (~100 LLM calls per analysis), even cloud costs are negligible. But local gives you zero dependency on external services.

[Download MemryLab](https://github.com/laadtushar/MemryLab/releases/latest) | [View All Providers](https://www.memrylab.com/docs/ai-providers)`,
  },
  {
    slug: "30-data-exports-you-didnt-know",
    title: "30+ Data Exports You Didn't Know You Could Download",
    metaTitle: "30+ Data Exports You Didn't Know You Could Download",
    excerpt: "Every major platform lets you export your data. Here's a complete list with direct links to download pages.",
    description: "Every major platform lets you export your data. A complete list of 30+ exports MemryLab can import, from Google Takeout to WhatsApp, with direct download links.",
    date: "2026-02-28",
    dateModified: "2026-10-10",
    author: "Tushar Laad",
    readTime: "7 min read",
    tags: ["guide", "data"],
    content: `# 30+ Data Exports You Didn't Know You Could Download

Under GDPR, CCPA, and similar regulations, every major platform must let you download your data. Most people never do. Here's every export MemryLab supports, with direct links.

## Social Media

| Platform | What You Get | Link |
|----------|-------------|------|
| **Twitter/X** | All tweets, DMs, likes, bookmarks | [twitter.com/settings/download_your_data](https://twitter.com/settings/download_your_data) |
| **Facebook** | Posts, messages, photos, reactions | [facebook.com/dyi](https://www.facebook.com/dyi) |
| **Instagram** | Posts, stories, messages, comments | [instagram.com > Settings > Your Activity > Download Your Information](https://www.instagram.com/download/request/) |
| **Reddit** | Posts, comments, saved items, messages | [reddit.com/settings/data-request](https://www.reddit.com/settings/data-request) |
| **LinkedIn** | Posts, messages, connections, profile | [linkedin.com/mypreferences/d/download-my-data](https://www.linkedin.com/mypreferences/d/download-my-data) |
| **Bluesky** | Posts, likes, follows | Export from Settings |
| **Mastodon** | Toots, media, followers | Export from Preferences |
| **TikTok** | Videos, comments, DMs | [tiktok.com > Settings > Download Your Data](https://www.tiktok.com/setting) |
| **Snapchat** | Memories, chats, story history | [accounts.snapchat.com/accounts/downloadmydata](https://accounts.snapchat.com/accounts/downloadmydata) |
| **Pinterest** | Pins, boards, messages | [pinterest.com/settings/privacy](https://www.pinterest.com/settings/privacy) |
| **Tumblr** | Posts, messages, likes | Export from Settings |

## Messaging

| Platform | What You Get | Link |
|----------|-------------|------|
| **WhatsApp** | Chat history (per chat) | In-app: Chat > More > Export Chat |
| **Telegram** | Full message history | Desktop app: Settings > Advanced > Export Data |
| **Discord** | DMs, server messages | [discord.com/channels/@me](https://discord.com/channels/@me) (request in Privacy) |
| **Slack** | Workspace messages, files | Workspace admin > Import/Export |
| **Signal** | Message backup | In-app backup (encrypted) |

## Notes & Writing

| Platform | What You Get | Link |
|----------|-------------|------|
| **Obsidian** | Already local! Just point MemryLab at your vault folder | N/A |
| **Notion** | All pages and databases | Notion > Settings > Export All Workspace Content |
| **Evernote** | All notes as .enex | Evernote > All Notes > Export |
| **Day One** | Journal entries as JSON | Day One > File > Export > JSON |
| **Substack** | All posts and subscriber data | Settings > Export |
| **Medium** | All stories and responses | [medium.com/me/export](https://medium.com/me/export) |

## Everything Else

| Platform | What You Get | Link |
|----------|-------------|------|
| **Google Takeout** | EVERYTHING: Gmail, Drive, Photos, YouTube, Search, Maps, Chrome | [takeout.google.com](https://takeout.google.com) |
| **Apple** | iCloud data, App Store, Apple ID | [privacy.apple.com](https://privacy.apple.com) |
| **Amazon** | Orders, searches, Alexa, Kindle | [amazon.com/gp/privacycentral](https://www.amazon.com/gp/privacycentral/dsar/preview.html) |
| **Spotify** | Listening history, playlists, searches | [spotify.com/account/privacy](https://www.spotify.com/account/privacy) |
| **YouTube** | Watch history, comments, subscriptions | Via Google Takeout |
| **Netflix** | Viewing history, ratings, searches | [netflix.com/account/getmyinfo](https://www.netflix.com/account/getmyinfo) |

## What To Do With All This Data

Once you've downloaded your exports:

1. **Import into MemryLab** — drop the ZIP files or folders into the Import view
2. **Run Analysis** — the 8-stage pipeline extracts themes, beliefs, and contradictions
3. **Explore** — use Timeline, Search, Ask, and Graph views to discover your evolution

Most people have 5-10 years of digital history they've never examined. The insights are waiting.

[Download MemryLab](https://github.com/laadtushar/MemryLab/releases/latest)`,
  },
  {
    slug: "contradiction-detection-how-ai-finds-belief-changes",
    title: "Contradiction Detection: How AI Finds Belief Changes You Never Noticed",
    metaTitle: "Contradiction Detection: How AI Finds Belief Changes",
    excerpt: "A deep dive into the embedding + LLM verification algorithm that detects when you've changed your mind.",
    description: "A deep dive into the embedding plus LLM verification approach that finds when you have changed your mind, and why pre-filtering keeps it fast on a laptop.",
    date: "2026-02-20",
    dateModified: "2026-10-10",
    author: "Tushar Laad",
    readTime: "10 min read",
    tags: ["technical", "AI"],
    content: `# Contradiction Detection: How AI Finds Belief Changes

## The Problem: O(n^2) Is Too Expensive

If you have 100 extracted beliefs, checking every pair for contradictions requires 4,950 LLM calls. At 500 beliefs, that's 124,750 calls. This is clearly prohibitive.

## The Solution: Embedding Pre-Filter + LLM Verification

MemryLab's contradiction detection uses a two-phase approach:

### Phase 1: Embedding Pre-Filter (O(n) calls)

1. Embed all active belief and preference facts using the embedding model
2. Compute pairwise cosine similarity
3. Retain only pairs with similarity > 0.7 (topically related)
4. Sort by similarity score, cap at k=20 pairs

**Key insight:** Two beliefs must be about the *same topic* before they can contradict each other. "I love hiking" and "I prefer tea over coffee" have low embedding similarity — they can't contradict regardless of content.

### Phase 2: LLM Verification (O(k) calls, k <= 20)

For each candidate pair, ask the LLM:

> "Are these two beliefs from the same person contradictory?"
> Belief A: "I value stability in my career"
> Belief B: "Playing it safe is the real risk"

The LLM returns:
- is_contradiction: true/false
- explanation: brief reasoning
- severity: minor/moderate/major

### Handling False Positives

A common false-positive boundary: "I enjoy working alone" and "I collaborate effectively in teams" are semantically related (high embedding similarity) but not contradictory. The LLM correctly classifies such pairs as compatible in 88% of cases.

## Results

On a benchmark of 50 synthetic belief pairs (25 true contradictions, 25 compatible):

| Method | LLM Calls | Precision | Recall | F1 |
|--------|-----------|-----------|--------|-----|
| Random pairing | 20 | 0.60 | 0.56 | 0.58 |
| All-pairs LLM | 1,275 | 0.80 | 0.72 | 0.76 |
| **Our method** (Llama 8B) | **20** | 0.80 | 0.72 | **0.76** |
| **Our method** (GPT-4o-mini) | **20** | 0.92 | 0.88 | **0.90** |

Our method matches all-pairs quality at **0.3% of the call count** — a 99.75% reduction.

## What It Finds

In a real case study across 14,653 documents:
- 127 belief-level facts extracted
- 4 genuine contradictions detected
- Most striking: a career-philosophy reversal spanning 20 months

The system also handles gradual shifts — beliefs that weren't contradictory at the time of writing but became contradictory given later statements.

## Try It Yourself

MemryLab's contradiction detector runs automatically during the analysis pipeline. Import your data, run analysis, and check the Insights view for detected contradictions.

[Download MemryLab](https://github.com/laadtushar/MemryLab/releases/latest) | [Read the Paper](https://github.com/laadtushar/MemryLab/tree/master/paper)`,
  },
  {
    slug: "google-takeout-to-self-knowledge",
    title: "From Google Takeout to Self-Knowledge: A Step-by-Step Guide",
    metaTitle: "From Google Takeout to Self-Knowledge: A Step-by-Step Guide",
    excerpt: "Download your Google data, import it into MemryLab, and discover insights about your digital life in 15 minutes.",
    description: "Download your Google data, import it into MemryLab, and discover insights about your digital life in about 15 minutes. A step-by-step Google Takeout guide.",
    date: "2026-02-10",
    dateModified: "2026-10-10",
    author: "Tushar Laad",
    readTime: "6 min read",
    tags: ["tutorial", "google"],
    content: `# From Google Takeout to Self-Knowledge

## Step 1: Request Your Google Data (2 minutes)

1. Go to [takeout.google.com](https://takeout.google.com)
2. Click "Deselect all" then select the data you want:
   - **Gmail** (emails and labels)
   - **My Activity** (search history, YouTube watches)
   - **Chrome** (bookmarks and history)
   - **Google Chat/Hangouts** (messages)
   - **Keep** (notes)
   - **YouTube** (comments, subscriptions, watch history)
3. Click "Next step"
4. Choose "Export once", ".zip" format, max file size 50 GB
5. Click "Create export"
6. Wait for the email (usually 1-24 hours for large exports)

## Step 2: Download and Import (5 minutes)

1. Download the ZIP file(s) from the email link
2. Open MemryLab
3. Click **Import** > **Auto-detect & Import**
4. Select the Google Takeout ZIP file
5. MemryLab auto-detects the Google Takeout format and starts importing

The import runs in the background — you can explore the app while it processes. For a typical 5-year Google account, expect:
- 2,000-10,000 documents
- 3-8 minutes import time
- ~50-200 MB of indexed data

## Step 3: Run Analysis (10-25 minutes)

After import completes, click **Run Analysis** in the completion banner. The 8-stage pipeline:

1. **Theme Extraction** — identifies recurring topics in your writing
2. **Sentiment Tracking** — emotional tone over time
3. **Belief Extraction** — explicit and implicit beliefs
4. **Entity Extraction** — people, places, organizations
5. **Insight Generation** — non-obvious patterns
6. **Contradiction Detection** — belief changes
7. **Evolution Arcs** — trends over time
8. **Narrative Generation** — prose summaries

## Step 4: Explore Your Data

### Timeline View
See your writing volume by month/week/day. Spot periods of high activity and quiet stretches.

### Ask View
Chat with your data: "What did I care about in 2022?" or "How have my views on career changed?"

### Knowledge Graph
Visualize connections between people, places, and concepts in your life.

### Evolution View
See how themes and sentiments shifted over time. Read AI-generated narratives about your personal growth.

## What Most People Discover

- **Theme persistence**: 2-3 topics dominate your thinking across years
- **Seasonal patterns**: writing volume and sentiment correlate with seasons
- **Unconscious shifts**: beliefs that changed gradually without your awareness
- **Connection patterns**: relationships that appear more in your writing than you'd expect

## Privacy Note

MemryLab is local by default: your Google data stays on your device unless you choose a cloud model. Importing and storing it happen on your machine. The AI analysis runs through your configured provider, which is local Ollama by default. If you pick a cloud API instead, the text it analyses is sent to that provider.

[Download MemryLab](https://github.com/laadtushar/MemryLab/releases/latest) | [See All Import Sources](https://www.memrylab.com/docs/import-sources)`,
  },
];
