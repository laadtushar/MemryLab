import Link from "next/link";

const providers = [
  {
    name: "Google Gemini",
    free: true,
    freeDetails: "1500 req/day on Gemini 1.5 Flash",
    url: "https://ai.google.dev",
    setup: `1. Go to ai.google.dev and sign in
2. Create an API key
3. In MemryLab: Settings → LLM → Select "Gemini"
4. Paste your API key and test connection`,
  },
  {
    name: "Groq",
    free: true,
    freeDetails: "Free tier with rate limits",
    url: "https://console.groq.com",
    setup: `1. Go to console.groq.com and create an account
2. Generate an API key from the dashboard
3. In MemryLab: Settings → LLM → Select "Groq"
4. Paste your API key and test connection`,
  },
  {
    name: "OpenRouter",
    free: true,
    freeDetails: "Free models available (Llama, Mistral, etc.)",
    url: "https://openrouter.ai",
    setup: `1. Go to openrouter.ai and create an account
2. Generate an API key
3. In MemryLab: Settings → LLM → Select "OpenRouter"
4. Paste your API key — free models are auto-selected`,
  },
  {
    name: "Cerebras",
    free: true,
    freeDetails: "Free tier, fastest inference",
    url: "https://cloud.cerebras.ai",
    setup: `1. Go to cloud.cerebras.ai and sign up
2. Create an API key from the dashboard
3. In MemryLab: Settings → LLM → Select "Cerebras"
4. Paste your API key and test connection`,
  },
  {
    name: "Mistral",
    free: true,
    freeDetails: "Free tier for small models",
    url: "https://console.mistral.ai",
    setup: `1. Go to console.mistral.ai and create an account
2. Generate an API key
3. In MemryLab: Settings → LLM → Select "Mistral"
4. Paste your API key and test connection`,
  },
  {
    name: "SambaNova",
    free: true,
    freeDetails: "Free tier with generous limits",
    url: "https://cloud.sambanova.ai",
    setup: `1. Go to cloud.sambanova.ai and sign up
2. Create an API key
3. In MemryLab: Settings → LLM → Select "SambaNova"
4. Paste your API key and test connection`,
  },
  {
    name: "Cohere",
    free: true,
    freeDetails: "Free trial API key",
    url: "https://dashboard.cohere.com",
    setup: `1. Go to dashboard.cohere.com and sign up
2. Copy your trial API key
3. In MemryLab: Settings → LLM → Select "Cohere"
4. Paste your API key and test connection`,
  },
  {
    name: "OpenAI",
    free: false,
    freeDetails: "Pay-per-use, starts at $0.002/1K tokens",
    url: "https://platform.openai.com",
    setup: `1. Go to platform.openai.com and sign in
2. Create an API key under API Keys
3. In MemryLab: Settings → LLM → Select "OpenAI"
4. Paste your API key and test connection`,
  },
  {
    name: "Custom (OpenAI-compatible)",
    free: false,
    freeDetails: "Any OpenAI-compatible endpoint (LM Studio, Ollama, etc.)",
    url: "",
    setup: `1. In MemryLab: Settings → LLM → Select "Custom"
2. Enter the base URL (e.g., http://localhost:1234/v1)
3. Enter API key if required (or leave blank for local)
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
        MemryLab supports 9 LLM providers including 8 with free tiers. You can
        switch providers at any time without losing data.
      </p>

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
                          className="hover:text-violet-400 transition"
                        >
                          {p.name}
                        </a>
                      ) : (
                        p.name
                      )}
                    </td>
                    <td className="py-3 pr-4">
                      {p.free ? (
                        <span className="px-2 py-0.5 rounded bg-green-900/30 text-green-400 text-xs font-medium">
                          Yes
                        </span>
                      ) : (
                        <span className="px-2 py-0.5 rounded bg-zinc-800 text-zinc-400 text-xs font-medium">
                          Paid
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
