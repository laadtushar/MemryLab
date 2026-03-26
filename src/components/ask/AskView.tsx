import { useState, useRef, useEffect, useCallback } from "react";
import {
  commands,
  type RagResponse,
  type ChatConversation,
  type ChatMessage,
  type RagSource,
} from "@/lib/tauri";
import {
  MessageCircle,
  Send,
  Clock,
  FileText,
  Loader2,
  Sparkles,
  Plus,
  Trash2,
} from "lucide-react";

interface LocalMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  sources?: RagSource[];
  timestamp: Date;
}

function relativeTime(timestamp: string): string {
  const date = new Date(timestamp + "Z");
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMin = Math.floor(diffMs / 60000);
  const diffHr = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHr / 24);

  if (diffMin < 1) return "just now";
  if (diffMin < 60) return `${diffMin}m ago`;
  if (diffHr < 24) return `${diffHr}h ago`;
  if (diffDay < 7) return `${diffDay}d ago`;
  return date.toLocaleDateString();
}

export function AskView() {
  const [input, setInput] = useState("");
  const [messages, setMessages] = useState<LocalMessage[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Chat history state
  const [conversations, setConversations] = useState<ChatConversation[]>([]);
  const [activeConversationId, setActiveConversationId] = useState<
    string | null
  >(null);
  const [hoveredConvId, setHoveredConvId] = useState<string | null>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(scrollToBottom, [messages]);

  // Load conversation list
  const loadConversations = useCallback(async () => {
    try {
      const convs = await commands.listConversations(50);
      setConversations(convs);
    } catch (e) {
      console.error("Failed to load conversations:", e);
    }
  }, []);

  useEffect(() => {
    loadConversations();
  }, [loadConversations]);

  // Load messages for a conversation
  const loadConversation = useCallback(async (convId: string) => {
    try {
      const msgs = await commands.getConversationMessages(convId);
      setMessages(
        msgs.map((m: ChatMessage) => ({
          id: m.id,
          role: m.role as "user" | "assistant",
          content: m.content,
          sources: m.sources.length > 0 ? m.sources : undefined,
          timestamp: new Date(m.created_at + "Z"),
        }))
      );
      setActiveConversationId(convId);
      setError(null);
    } catch (e) {
      console.error("Failed to load conversation:", e);
    }
  }, []);

  const handleNewChat = () => {
    setActiveConversationId(null);
    setMessages([]);
    setError(null);
    setInput("");
  };

  const handleDeleteConversation = async (
    e: React.MouseEvent,
    convId: string
  ) => {
    e.stopPropagation();
    try {
      await commands.deleteConversation(convId);
      if (activeConversationId === convId) {
        handleNewChat();
      }
      loadConversations();
    } catch (err) {
      console.error("Failed to delete conversation:", err);
    }
  };

  const handleAsk = async () => {
    const question = input.trim();
    if (!question || loading) return;

    setInput("");
    setError(null);

    const userMsg: LocalMessage = {
      id: crypto.randomUUID(),
      role: "user",
      content: question,
      timestamp: new Date(),
    };
    setMessages((prev) => [...prev, userMsg]);
    setLoading(true);

    try {
      const response: RagResponse = await commands.ask(
        question,
        activeConversationId ?? undefined
      );
      const assistantMsg: LocalMessage = {
        id: crypto.randomUUID(),
        role: "assistant",
        content: response.answer,
        sources: response.sources,
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, assistantMsg]);

      // Update active conversation
      if (response.conversation_id) {
        setActiveConversationId(response.conversation_id);
      }

      // Refresh sidebar
      loadConversations();
    } catch (e) {
      setError(String(e));
      const errorMsg: LocalMessage = {
        id: crypto.randomUUID(),
        role: "assistant",
        content:
          "I couldn't answer that question. Make sure your LLM provider is running and you have documents imported.",
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, errorMsg]);
    }
    setLoading(false);
  };

  return (
    <div className="flex h-full">
      {/* Conversation sidebar */}
      <div className="w-64 shrink-0 border-r border-border bg-sidebar flex flex-col">
        <div className="p-3 border-b border-border">
          <button
            onClick={handleNewChat}
            className="w-full flex items-center justify-center gap-2 rounded-md bg-primary px-3 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
          >
            <Plus size={16} />
            New Chat
          </button>
        </div>
        <div className="flex-1 overflow-y-auto">
          {conversations.length === 0 && (
            <div className="px-3 py-6 text-center text-xs text-muted-foreground">
              No conversations yet
            </div>
          )}
          {conversations.map((conv) => (
            <button
              key={conv.id}
              onClick={() => loadConversation(conv.id)}
              onMouseEnter={() => setHoveredConvId(conv.id)}
              onMouseLeave={() => setHoveredConvId(null)}
              className={`w-full text-left px-3 py-2.5 border-b border-border/50 flex items-start gap-2 transition-colors group ${
                activeConversationId === conv.id
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:bg-accent/50 hover:text-foreground"
              }`}
            >
              <MessageCircle size={14} className="shrink-0 mt-0.5" />
              <div className="flex-1 min-w-0">
                <div className="text-xs font-medium truncate">
                  {conv.title}
                </div>
                <div className="text-[10px] text-muted-foreground mt-0.5">
                  {conv.message_count} messages &middot;{" "}
                  {relativeTime(conv.updated_at)}
                </div>
              </div>
              {hoveredConvId === conv.id && (
                <button
                  onClick={(e) => handleDeleteConversation(e, conv.id)}
                  className="shrink-0 p-0.5 rounded text-muted-foreground hover:text-destructive transition-colors"
                  title="Delete conversation"
                >
                  <Trash2 size={12} />
                </button>
              )}
            </button>
          ))}
        </div>
      </div>

      {/* Chat area */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Header */}
        <div className="border-b border-border px-6 py-4">
          <h1 className="text-2xl font-semibold flex items-center gap-2">
            <Sparkles size={24} className="text-primary" /> Ask Your Memory
          </h1>
          <p className="text-sm text-muted-foreground mt-1">
            Ask questions about your personal documents. Answers are grounded in
            your own writing.
          </p>
        </div>

        {/* Messages */}
        <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4">
          {messages.length === 0 && (
            <div className="flex flex-col items-center justify-center h-full text-center space-y-4">
              <MessageCircle
                size={48}
                className="text-muted-foreground/30"
              />
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
                      onClick={() => setInput(q)}
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
              onKeyDown={(e) =>
                e.key === "Enter" && !e.shiftKey && handleAsk()
              }
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
    </div>
  );
}
