import { useState, useRef, useEffect } from "react";
import { commands, type RagResponse } from "@/lib/tauri";
import { MessageCircle, Send, Clock, FileText, Loader2, Sparkles } from "lucide-react";

interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
  sources?: RagResponse["sources"];
  timestamp: Date;
}

export function AskView() {
  const [input, setInput] = useState("");
  const [messages, setMessages] = useState<Message[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(scrollToBottom, [messages]);

  const handleAsk = async () => {
    const question = input.trim();
    if (!question || loading) return;

    setInput("");
    setError(null);

    const userMsg: Message = {
      id: crypto.randomUUID(),
      role: "user",
      content: question,
      timestamp: new Date(),
    };
    setMessages((prev) => [...prev, userMsg]);
    setLoading(true);

    try {
      const response = await commands.ask(question);
      const assistantMsg: Message = {
        id: crypto.randomUUID(),
        role: "assistant",
        content: response.answer,
        sources: response.sources,
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, assistantMsg]);
    } catch (e) {
      setError(String(e));
      const errorMsg: Message = {
        id: crypto.randomUUID(),
        role: "assistant",
        content: "I couldn't answer that question. Make sure Ollama is running and you have documents imported.",
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, errorMsg]);
    }
    setLoading(false);
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="border-b border-border px-6 py-4">
        <h1 className="text-2xl font-semibold flex items-center gap-2">
          <Sparkles size={24} className="text-primary" /> Ask Your Memory
        </h1>
        <p className="text-sm text-muted-foreground mt-1">
          Ask questions about your personal documents. Answers are grounded in your own writing.
        </p>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4">
        {messages.length === 0 && (
          <div className="flex flex-col items-center justify-center h-full text-center space-y-4">
            <MessageCircle size={48} className="text-muted-foreground/30" />
            <div>
              <p className="text-lg font-medium text-muted-foreground">
                Ask anything about your documents
              </p>
              <div className="mt-3 space-y-2">
                {[
                  "How has my view on career changed over time?",
                  "What was I writing about last summer?",
                  "What are my core beliefs?",
                  "When did I start thinking about moving?",
                ].map((q) => (
                  <button
                    key={q}
                    onClick={() => {
                      setInput(q);
                    }}
                    className="block w-full text-left rounded-lg border border-border bg-card px-4 py-2.5 text-sm text-muted-foreground hover:border-primary/50 hover:text-foreground transition-colors"
                  >
                    {q}
                  </button>
                ))}
              </div>
            </div>
          </div>
        )}

        {messages.map((msg) => (
          <div
            key={msg.id}
            className={`flex ${msg.role === "user" ? "justify-end" : "justify-start"}`}
          >
            <div
              className={`max-w-[80%] rounded-lg px-4 py-3 ${
                msg.role === "user"
                  ? "bg-primary text-primary-foreground"
                  : "bg-card border border-border"
              }`}
            >
              <p className="text-sm whitespace-pre-wrap">{msg.content}</p>

              {/* Source citations */}
              {msg.sources && msg.sources.length > 0 && (
                <div className="mt-3 pt-3 border-t border-border/50 space-y-1.5">
                  <p className="text-xs font-medium text-muted-foreground">
                    Sources ({msg.sources.length}):
                  </p>
                  {msg.sources.map((src, i) => (
                    <div
                      key={src.chunk_id}
                      className="rounded bg-background/50 px-2.5 py-1.5 text-xs"
                    >
                      <div className="flex items-center gap-2 text-muted-foreground mb-0.5">
                        <span className="font-mono">[{i + 1}]</span>
                        {src.timestamp && (
                          <span className="flex items-center gap-0.5">
                            <Clock size={10} />
                            {new Date(src.timestamp).toLocaleDateString()}
                          </span>
                        )}
                        <span className="flex items-center gap-0.5">
                          <FileText size={10} />
                          {src.score.toFixed(3)}
                        </span>
                      </div>
                      <p className="text-muted-foreground/80 line-clamp-2">
                        {src.text_snippet}
                      </p>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        ))}

        {loading && (
          <div className="flex justify-start">
            <div className="rounded-lg bg-card border border-border px-4 py-3">
              <Loader2 size={16} className="animate-spin text-primary" />
            </div>
          </div>
        )}

        {error && !loading && (
          <div className="rounded-md bg-destructive/10 border border-destructive/20 px-4 py-2 text-sm text-destructive">
            {error}
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <div className="border-t border-border px-6 py-4">
        <div className="flex gap-2">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && !e.shiftKey && handleAsk()}
            placeholder="Ask a question about your documents..."
            disabled={loading}
            className="flex-1 rounded-md border border-input bg-background px-4 py-2.5 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring disabled:opacity-50"
          />
          <button
            onClick={handleAsk}
            disabled={loading || !input.trim()}
            className="rounded-md bg-primary px-4 py-2.5 text-primary-foreground hover:bg-primary/90 disabled:opacity-50 transition-colors"
          >
            <Send size={16} />
          </button>
        </div>
      </div>
    </div>
  );
}
