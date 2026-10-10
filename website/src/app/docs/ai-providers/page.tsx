import Link from "next/link";
import { pageMetadata } from "@/lib/seo";

export const metadata = pageMetadata({
  title: "AI Providers for MemryLab: Ollama, Gemini, Groq and More",
  description:
    "Set up one of nine LLM providers in MemryLab, eight with free tiers, including local Ollama. Mix a cloud model for analysis with private local embeddings.",
  path: "/docs/ai-providers",
});

type Tier = "free" | "paid" | "varies";

// The nine presets in Settings > AI Provider, plus the custom endpoint. Free-tier limits are
// the ones the app lists for each preset (src-tauri/src/commands/provider_config.rs).
const providers: { name: string; tier: Tier; freeDetails: string; url: string; setup: string }[] = [
  {
    name: "Ollama (Local)",
    tier: "free",
    freeDetails: "Runs on your own hardware, no limits and no internet needed",
    url: "https://ollama.com/download",
    setup: `1. Install Ollama from ollama.com/download
2. Pull a model: ollama pull llama3.1:8b
3. Pull the embedding model: ollama pull nomic-embed-text
4. In MemryLab: Settings → AI Provider → select "Ollama (Local)"
5. No API key is needed`,
  },
  {
    name: "OpenRouter",
    tier: "free",
    freeDetails: "Free models from several labs, about 20 requests/min and 200/day per model",
    url: "https://openrouter.ai",
    setup: `1. Go to openrouter.ai and create an account
2. Generate an API key
3. In MemryLab: Settings → AI Provider → select "OpenRouter"
4. Paste your API key — a free model is pre-selected`,
  },
  {
    name: "Groq",
    tier: "free",
    freeDetails: "Free tier: 30 requests/min, about 1,000 requests/day on the 70B model",
    url: "https://console.groq.com",
    setup: `1. Go to console.groq.com and create an account
2. Generate an API key from the dashboard
3. In MemryLab: Settings → AI Provider → select "Groq"
4. Paste your API key and test connection`,
  },
  {
    name: "Google Gemini",
    tier: "free",
    freeDetails: "Free tier: 10 requests/min on Flash, 5 on Pro, no credit card. Includes embeddings",
    url: "https://aistudio.google.com",
    setup: `1. Go to aistudio.google.com and sign in
2. Create an API key
3. In MemryLab: Settings → AI Provider → select "Google Gemini"
4. Paste your API key and test connection`,
  },
  {
    name: "Cerebras",
    tier: "free",
    freeDetails: "Free tier: 1M tokens/day, very fast inference",
    url: "https://cloud.cerebras.ai",
    setup: `1. Go to cloud.cerebras.ai and sign up
2. Create an API key from the dashboard
3. In MemryLab: Settings → AI Provider → select "Cerebras"
4. Paste your API key and test connection`,
  },
  {
    name: "Mistral",
    tier: "free",
    freeDetails: "Free experiment plan: 1 request/sec, up to 1B tokens/month. Includes embeddings",
    url: "https://console.mistral.ai",
    setup: `1. Go to console.mistral.ai and create an account
2. Generate an API key
3. In MemryLab: Settings → AI Provider → select "Mistral"
4. Paste your API key and test connection`,
  },
  {
    name: "SambaNova",
    tier: "free",
    freeDetails: "Free tier: 10-30 requests/min depending on the model",
    url: "https://cloud.sambanova.ai",
    setup: `1. Go to cloud.sambanova.ai and sign up
2. Create an API key
3. In MemryLab: Settings → AI Provider → select "SambaNova"
4. Paste your API key and test connection`,
  },
  {
    name: "Cohere",
    tier: "free",
    freeDetails: "Free tier: 1,000 API calls/month. Includes embeddings",
    url: "https://dashboard.cohere.com",
    setup: `1. Go to dashboard.cohere.com and sign up
2. Copy your trial API key
3. In MemryLab: Settings → AI Provider → select "Cohere"
4. Paste your API key and test connection`,
  },
  {
    name: "Claude (Anthropic)",
    tier: "paid",
    freeDetails: "Pay-per-use API key, no free tier",
    url: "https://console.anthropic.com",
    setup: `1. Go to console.anthropic.com and sign in
2. Create an API key
3. In MemryLab: Settings → AI Provider → select "Claude (Anthropic)"
4. Paste your API key and test connection`,
  },
  {
    name: "Custom (OpenAI-compatible)",
    tier: "varies",
    freeDetails: "Any OpenAI-compatible endpoint (LM Studio, vLLM, OpenAI and others)",
    url: "",
    setup: `1. In MemryLab: Settings → AI Provider → Custom OpenAI-compatible endpoint
2. Enter the base URL (e.g., http://localhost:1234/v1)
3. Enter an API key if required (or leave blank for local)
4. Select a model name and test connection`,
  },
];

export default function AIProvidersPage() {
  return (
    <div>
      <div className="flex items-center gap-2 text-sm text-zinc-500 mb-8">
        <Link href="/docs" className="hover:text-white transition">
          Docs
        </Link>
        <span>/</span>
        <span className="text-white">AI Providers</span>
      </div>

      <h1 className="text-4xl font-bold mb-6">AI Providers</h1>
      <p className="text-zinc-400 text-lg mb-8">
        MemryLab has 9 provider presets, 8 of them with free tiers (Ollama runs
        locally and needs no key), plus a custom OpenAI-compatible endpoint. You
        can switch providers at any time without losing data. LLM and embedding
        providers can be configured independently — for example, use Gemini for
        analysis and Ollama for private local embeddings.
      </p>

      {/* Recommended models */}
      <section className="mb-10">
        <h2 className="text-2xl font-semibold text-white mb-4">Recommended Local Models (Ollama)</h2>
        <p className="text-zinc-400 mb-4">
          For local inference, pick models based on your GPU VRAM. Embedding model is always <code className="text-violet-400">nomic-embed-text</code>.
        </p>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-zinc-800 text-left">
                <th className="py-3 pr-4 text-zinc-400 font-medium">VRAM</th>
                <th className="py-3 pr-4 text-zinc-400 font-medium">LLM Model</th>
                <th className="py-3 pr-4 text-zinc-400 font-medium">Speed</th>
                <th className="py-3 text-zinc-400 font-medium">Install</th>
              </tr>
            </thead>
            <tbody>
              <tr className="border-b border-zinc-800/50">
                <td className="py-3 pr-4 text-white">4 GB</td>
                <td className="py-3 pr-4 font-mono text-violet-400">llama3.2:3b</td>
                <td className="py-3 pr-4">~40 tok/s</td>
                <td className="py-3 font-mono text-xs">ollama pull llama3.2:3b</td>
              </tr>
              <tr className="border-b border-zinc-800/50">
                <td className="py-3 pr-4 text-white">8 GB</td>
                <td className="py-3 pr-4 font-mono text-violet-400">llama3.1:8b</td>
                <td className="py-3 pr-4">~35 tok/s</td>
                <td className="py-3 font-mono text-xs">ollama pull llama3.1:8b</td>
              </tr>
              <tr className="border-b border-zinc-800/50 bg-zinc-900/30">
                <td className="py-3 pr-4 text-white font-medium">12 GB ★</td>
                <td className="py-3 pr-4 font-mono text-violet-400">qwen2.5:14b-instruct-q5_K_M</td>
                <td className="py-3 pr-4">~25 tok/s</td>
                <td className="py-3 font-mono text-xs">ollama pull qwen2.5:14b-instruct-q5_K_M</td>
              </tr>
              <tr className="border-b border-zinc-800/50">
                <td className="py-3 pr-4 text-white">16 GB+</td>
                <td className="py-3 pr-4 font-mono text-violet-400">qwen2.5:32b-instruct-q4_K_M</td>
                <td className="py-3 pr-4">~15 tok/s</td>
                <td className="py-3 font-mono text-xs">ollama pull qwen2.5:32b-instruct-q4_K_M</td>
              </tr>
            </tbody>
          </table>
        </div>
        <p className="text-zinc-500 text-xs mt-2">
          Larger models produce better belief extraction and contradiction detection. For no-GPU setups, use Gemini Flash (free) or Groq (free) as cloud providers.
        </p>
      </section>

      {/* Separate providers tip */}
      <section className="mb-10 p-4 rounded-lg border border-violet-500/20 bg-violet-500/5">
        <h3 className="text-lg font-semibold text-white mb-2">💡 Pro Tip: Mix Providers</h3>
        <p className="text-zinc-400 text-sm">
          In Settings → Embedding Provider, you can use a <strong className="text-white">different provider for embeddings vs LLM</strong>.
          The best privacy setup: use <strong className="text-white">Gemini Flash</strong> (free, fast) for analysis + <strong className="text-white">Ollama</strong> (local) for embeddings.
          This way your search index stays fully private while analysis uses a powerful cloud model.
        </p>
      </section>

      <div className="space-y-8 text-zinc-300 leading-relaxed">
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            Quick Comparison
          </h2>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-zinc-800 text-left">
                  <th className="py-3 pr-4 text-zinc-400 font-medium">
                    Provider
                  </th>
                  <th className="py-3 pr-4 text-zinc-400 font-medium">
                    Free Tier
                  </th>
                  <th className="py-3 text-zinc-400 font-medium">Details</th>
                </tr>
              </thead>
              <tbody>
                {providers.map((p, i) => (
                  <tr
                    key={i}
                    className="border-b border-zinc-800/50 hover:bg-zinc-900/30"
                  >
                    <td className="py-3 pr-4 text-white font-medium">
                      {p.url ? (
                        <a
                          href={p.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="hover:text-violet-400 transition"
                        >
                          {p.name}
                        </a>
                      ) : (
                        p.name
                      )}
                    </td>
                    <td className="py-3 pr-4">
                      {p.tier === "free" ? (
                        <span className="px-2 py-0.5 rounded bg-green-900/30 text-green-400 text-xs font-medium">
                          Yes
                        </span>
                      ) : (
                        <span className="px-2 py-0.5 rounded bg-zinc-800 text-zinc-400 text-xs font-medium">
                          {p.tier === "paid" ? "Paid" : "Varies"}
                        </span>
                      )}
                    </td>
                    <td className="py-3 text-zinc-400">{p.freeDetails}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        {/* Individual provider setup */}
        {providers.map((p, i) => (
          <section key={i}>
            <h2 className="text-2xl font-semibold text-white mb-4">
              {p.name}
            </h2>
            <pre className="bg-zinc-900 rounded-lg p-4 text-sm overflow-x-auto mb-4">
              <code>{p.setup}</code>
            </pre>
            {p.url && (
              <p className="text-sm">
                <a
                  href={p.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-violet-400 hover:text-violet-300 underline"
                >
                  {p.url}
                </a>
              </p>
            )}
          </section>
        ))}

        {/* API Key Security */}
        <section>
          <h2 className="text-2xl font-semibold text-white mb-4">
            API Key Security
          </h2>
          <p className="mb-4">
            MemryLab stores API keys in your operating system&apos;s secure
            keychain — never in plain text files, environment variables, or the
            SQLite database.
          </p>
          <ul className="list-disc list-inside space-y-2 text-zinc-400">
            <li>
              <strong className="text-white">Windows:</strong> Windows
              Credential Manager
            </li>
            <li>
              <strong className="text-white">macOS:</strong> Keychain Access
            </li>
            <li>
              <strong className="text-white">Linux:</strong> libsecret (GNOME
              Keyring / KWallet)
            </li>
          </ul>
        </section>
      </div>

      <div className="flex items-center justify-between mt-16 pt-8 border-t border-zinc-800">
        <Link
          href="/docs/import-sources"
          className="text-sm text-zinc-500 hover:text-white transition"
        >
          &larr; Import Sources
        </Link>
        <Link
          href="/docs/architecture"
          className="text-sm text-violet-400 hover:text-violet-300 transition"
        >
          Architecture &rarr;
        </Link>
      </div>
    </div>
  );
}
