"use client";

import { useState } from "react";
import {
  BarChart3,
  GitBranch,
  MessageCircle,
  Upload,
  Search,
  Brain,
  Clock,
  Settings,
  BookOpen,
  Sparkles,
} from "lucide-react";

const tabs = [
  { id: "timeline", label: "Timeline", icon: BarChart3 },
  { id: "graph", label: "Graph Explorer", icon: GitBranch },
  { id: "ask", label: "Ask / Chat", icon: MessageCircle },
  { id: "import", label: "Import", icon: Upload },
] as const;

type TabId = (typeof tabs)[number]["id"];

const sidebarItems = [
  { icon: Clock, label: "Timeline" },
  { icon: Search, label: "Search" },
  { icon: MessageCircle, label: "Ask" },
  { icon: Brain, label: "Insights" },
  { icon: Sparkles, label: "Evolution" },
  { icon: Upload, label: "Import" },
  { icon: BookOpen, label: "Memory" },
  { icon: GitBranch, label: "Graph" },
  { icon: Settings, label: "Settings" },
];

/* ---------- Timeline mock ---------- */
function TimelineView() {
  const months = [
    "Jan",
    "Feb",
    "Mar",
    "Apr",
    "May",
    "Jun",
    "Jul",
    "Aug",
    "Sep",
    "Oct",
    "Nov",
    "Dec",
  ];
  const heights = [45, 70, 55, 90, 60, 80, 95, 50, 75, 85, 40, 65];
  const colors = [
    "bg-violet-500",
    "bg-blue-500",
    "bg-violet-400",
    "bg-blue-400",
    "bg-violet-500",
    "bg-cyan-500",
    "bg-violet-400",
    "bg-blue-500",
    "bg-cyan-400",
    "bg-violet-500",
    "bg-blue-400",
    "bg-cyan-500",
  ];

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-4 pt-3 pb-2">
        <div className="flex items-center gap-2">
          <span className="text-sm font-semibold text-zinc-200">
            Timeline
          </span>
          <span className="text-xs text-zinc-500">2023 - 2025</span>
        </div>
        <div className="flex gap-1">
          <span className="px-2 py-0.5 text-[10px] rounded bg-violet-600/30 text-violet-300 border border-violet-500/20">
            Year
          </span>
          <span className="px-2 py-0.5 text-[10px] rounded bg-zinc-800 text-zinc-400 border border-zinc-700">
            Month
          </span>
          <span className="px-2 py-0.5 text-[10px] rounded bg-zinc-800 text-zinc-400 border border-zinc-700">
            Week
          </span>
        </div>
      </div>

      {/* Chart area */}
      <div className="flex-1 flex items-end gap-[6px] px-4 pb-1 pt-2">
        {months.map((m, i) => (
          <div key={m} className="flex-1 flex flex-col items-center gap-1">
            <div
              className={`w-full rounded-t ${colors[i]} opacity-80 transition-all duration-500`}
              style={{ height: `${heights[i]}%`, minHeight: 8 }}
            />
            <span className="text-[9px] text-zinc-500">{m}</span>
          </div>
        ))}
      </div>

      {/* Summary row */}
      <div className="flex items-center gap-4 px-4 py-2 border-t border-zinc-800 text-[10px] text-zinc-500">
        <span>
          <strong className="text-zinc-300">1,247</strong> entries
        </span>
        <span>
          <strong className="text-zinc-300">23</strong> themes
        </span>
        <span>
          <strong className="text-zinc-300">156</strong> entities
        </span>
      </div>
    </div>
  );
}

/* ---------- Graph mock ---------- */
function GraphView() {
  const nodes = [
    { x: 50, y: 40, r: 20, color: "#8b5cf6", label: "You" },
    { x: 25, y: 25, r: 12, color: "#3b82f6", label: "Travel" },
    { x: 78, y: 28, r: 14, color: "#06b6d4", label: "Work" },
    { x: 35, y: 68, r: 11, color: "#f59e0b", label: "Music" },
    { x: 72, y: 65, r: 13, color: "#10b981", label: "Friends" },
    { x: 15, y: 50, r: 9, color: "#ec4899", label: "Books" },
    { x: 88, y: 50, r: 10, color: "#6366f1", label: "Health" },
    { x: 50, y: 82, r: 10, color: "#f97316", label: "Coding" },
    { x: 60, y: 15, r: 8, color: "#14b8a6", label: "Ideas" },
  ];

  const edges = [
    [0, 1],
    [0, 2],
    [0, 3],
    [0, 4],
    [0, 5],
    [0, 6],
    [0, 7],
    [1, 8],
    [2, 4],
    [2, 6],
    [3, 7],
    [4, 7],
    [5, 8],
  ];

  return (
    <div className="relative h-full w-full">
      <svg
        viewBox="0 0 100 100"
        className="w-full h-full"
        preserveAspectRatio="xMidYMid meet"
      >
        {/* Edges */}
        {edges.map(([a, b], i) => (
          <line
            key={i}
            x1={nodes[a].x}
            y1={nodes[a].y}
            x2={nodes[b].x}
            y2={nodes[b].y}
            stroke="#3f3f46"
            strokeWidth="0.4"
            opacity="0.6"
          />
        ))}
        {/* Nodes */}
        {nodes.map((n, i) => (
          <g key={i}>
            <circle
              cx={n.x}
              cy={n.y}
              r={n.r / 2.5}
              fill={n.color}
              opacity="0.85"
            />
            <text
              x={n.x}
              y={n.y + n.r / 2.5 + 4}
              textAnchor="middle"
              className="fill-zinc-400"
              fontSize="3"
            >
              {n.label}
            </text>
          </g>
        ))}
      </svg>
      {/* Floating detail panel */}
      <div className="absolute top-3 right-3 w-28 bg-zinc-900/90 border border-zinc-700 rounded-lg p-2 text-[10px]">
        <div className="flex items-center gap-1 mb-1">
          <div className="w-2 h-2 rounded-full bg-violet-500" />
          <span className="text-zinc-200 font-medium">You</span>
        </div>
        <div className="text-zinc-500 space-y-0.5">
          <div>Connections: 7</div>
          <div>Mentions: 1,247</div>
          <div>Type: Person</div>
        </div>
      </div>
    </div>
  );
}

/* ---------- Ask/Chat mock ---------- */
function AskView() {
  return (
    <div className="flex flex-col h-full">
      <div className="flex-1 overflow-hidden px-4 py-3 space-y-3">
        {/* User message */}
        <div className="flex justify-end">
          <div className="max-w-[70%] rounded-2xl rounded-br-sm bg-violet-600/20 border border-violet-500/20 px-3 py-2 text-xs text-zinc-200">
            How has my interest in music changed over the past 2 years?
          </div>
        </div>
        {/* AI response */}
        <div className="flex justify-start">
          <div className="max-w-[80%] rounded-2xl rounded-bl-sm bg-zinc-800 border border-zinc-700 px-3 py-2 text-xs text-zinc-300 space-y-2">
            <p>
              Based on your entries, your music interests shifted
              significantly:
            </p>
            <ul className="list-disc list-inside text-zinc-400 space-y-0.5 text-[11px]">
              <li>
                <strong className="text-zinc-300">Early 2023:</strong>{" "}
                Mostly indie rock and lo-fi playlists
              </li>
              <li>
                <strong className="text-zinc-300">Mid 2024:</strong>{" "}
                Started exploring jazz and ambient
              </li>
              <li>
                <strong className="text-zinc-300">Now:</strong>{" "}
                Heavy focus on classical and film scores
              </li>
            </ul>
            <div className="flex items-center gap-2 pt-1 border-t border-zinc-700 text-[10px] text-zinc-500">
              <span>Sources: 23 entries</span>
              <span className="w-px h-3 bg-zinc-700" />
              <span>Confidence: High</span>
            </div>
          </div>
        </div>
        {/* Suggested questions */}
        <div className="flex flex-wrap gap-1">
          {[
            "What triggered the shift?",
            "Top artists per year",
            "Mood patterns",
          ].map((q) => (
            <span
              key={q}
              className="px-2 py-1 text-[10px] rounded-full bg-zinc-800 border border-zinc-700 text-zinc-400"
            >
              {q}
            </span>
          ))}
        </div>
      </div>
      {/* Input bar */}
      <div className="px-4 py-2 border-t border-zinc-800">
        <div className="flex items-center gap-2 bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2">
          <span className="text-xs text-zinc-500 flex-1">
            Ask your memory anything...
          </span>
          <div className="w-5 h-5 rounded bg-violet-600 flex items-center justify-center">
            <MessageCircle size={10} className="text-white" />
          </div>
        </div>
      </div>
    </div>
  );
}

/* ---------- Import mock ---------- */
function ImportView() {
  const sources = [
    { name: "Google", color: "bg-red-500" },
    { name: "WhatsApp", color: "bg-green-500" },
    { name: "Twitter", color: "bg-sky-500" },
    { name: "Reddit", color: "bg-orange-500" },
    { name: "Obsidian", color: "bg-purple-500" },
    { name: "Notion", color: "bg-zinc-400" },
    { name: "Telegram", color: "bg-blue-500" },
    { name: "Discord", color: "bg-indigo-500" },
    { name: "Spotify", color: "bg-green-400" },
    { name: "YouTube", color: "bg-red-600" },
    { name: "Facebook", color: "bg-blue-600" },
    { name: "Instagram", color: "bg-pink-500" },
    { name: "Day One", color: "bg-yellow-500" },
    { name: "Slack", color: "bg-fuchsia-500" },
    { name: "Evernote", color: "bg-emerald-600" },
    { name: "LinkedIn", color: "bg-blue-700" },
  ];

  return (
    <div className="flex flex-col h-full">
      <div className="px-4 pt-3 pb-2">
        <span className="text-sm font-semibold text-zinc-200">
          Import Sources
        </span>
        <p className="text-[10px] text-zinc-500 mt-0.5">
          Drop a ZIP/folder or pick a source below
        </p>
      </div>
      {/* Drop zone */}
      <div className="mx-4 mb-3 border-2 border-dashed border-zinc-700 rounded-lg py-4 flex flex-col items-center justify-center">
        <Upload size={16} className="text-zinc-500 mb-1" />
        <span className="text-[10px] text-zinc-500">
          Drag and drop files here
        </span>
      </div>
      {/* Source grid */}
      <div className="flex-1 px-4 pb-3">
        <div className="grid grid-cols-4 gap-2">
          {sources.map((s) => (
            <div
              key={s.name}
              className="flex flex-col items-center gap-1 p-2 rounded-lg bg-zinc-800/50 border border-zinc-700/50 hover:border-zinc-600 transition"
            >
              <div
                className={`w-5 h-5 rounded ${s.color} flex items-center justify-center text-[8px] font-bold text-white`}
              >
                {s.name[0]}
              </div>
              <span className="text-[9px] text-zinc-400">{s.name}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

/* ---------- Main AppPreview ---------- */
export default function AppPreview() {
  const [activeTab, setActiveTab] = useState<TabId>("timeline");

  const views: Record<TabId, React.ReactNode> = {
    timeline: <TimelineView />,
    graph: <GraphView />,
    ask: <AskView />,
    import: <ImportView />,
  };

  const activeTabIndex = tabs.findIndex((t) => t.id === activeTab);
  const activeSidebarIndex = (() => {
    switch (activeTab) {
      case "timeline":
        return 0;
      case "graph":
        return 7;
      case "ask":
        return 2;
      case "import":
        return 5;
    }
  })();

  return (
    <div className="rounded-xl overflow-hidden border border-zinc-700/50 shadow-2xl shadow-violet-500/5 bg-zinc-950">
      {/* macOS title bar */}
      <div className="flex items-center gap-2 px-4 py-3 bg-zinc-900 border-b border-zinc-800">
        <div className="flex items-center gap-1.5">
          <div className="w-3 h-3 rounded-full bg-red-500/80" />
          <div className="w-3 h-3 rounded-full bg-yellow-500/80" />
          <div className="w-3 h-3 rounded-full bg-green-500/80" />
        </div>
        <div className="flex-1 text-center text-xs text-zinc-500 font-medium">
          MemryLab
        </div>
        <div className="w-12" /> {/* Balance */}
      </div>

      {/* Tab bar */}
      <div className="flex items-center gap-0 px-4 bg-zinc-900/80 border-b border-zinc-800">
        {tabs.map((tab, i) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex items-center gap-1.5 px-4 py-2.5 text-xs font-medium transition-colors relative ${
              activeTab === tab.id
                ? "text-violet-300"
                : "text-zinc-500 hover:text-zinc-300"
            }`}
          >
            <tab.icon size={13} />
            {tab.label}
            {activeTab === tab.id && (
              <div className="absolute bottom-0 left-2 right-2 h-0.5 bg-violet-500 rounded-full" />
            )}
          </button>
        ))}
      </div>

      {/* Main area: sidebar + content */}
      <div className="flex" style={{ height: 380 }}>
        {/* Sidebar */}
        <div className="w-12 bg-zinc-900/50 border-r border-zinc-800 flex flex-col items-center py-3 gap-1">
          {sidebarItems.map((item, i) => (
            <button
              key={item.label}
              className={`w-8 h-8 rounded-lg flex items-center justify-center transition-colors ${
                i === activeSidebarIndex
                  ? "bg-violet-600/20 text-violet-400"
                  : "text-zinc-600 hover:text-zinc-400"
              }`}
              title={item.label}
            >
              <item.icon size={15} />
            </button>
          ))}
        </div>

        {/* Content */}
        <div className="flex-1 min-w-0">{views[activeTab]}</div>
      </div>
    </div>
  );
}
