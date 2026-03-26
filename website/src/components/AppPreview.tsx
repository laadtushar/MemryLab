"use client";

import { useState } from "react";
import {
  BarChart3, GitBranch, MessageCircle, Upload, Search, Brain,
  Clock, Settings, BookOpen, Sparkles, TrendingUp, Shield,
  Activity, Zap, Database, ChevronDown, Filter, ExternalLink,
} from "lucide-react";

type ViewId = "timeline" | "search" | "ask" | "insights" | "evolution" | "import" | "memory" | "graph" | "settings";

const sidebarItems: { id: ViewId; icon: typeof Clock; label: string }[] = [
  { id: "timeline", icon: Clock, label: "Timeline" },
  { id: "search", icon: Search, label: "Search" },
  { id: "ask", icon: MessageCircle, label: "Ask" },
  { id: "insights", icon: Brain, label: "Insights" },
  { id: "evolution", icon: TrendingUp, label: "Evolution" },
  { id: "import", icon: Upload, label: "Import" },
  { id: "memory", icon: BookOpen, label: "Memory" },
  { id: "graph", icon: GitBranch, label: "Graph" },
  { id: "settings", icon: Settings, label: "Settings" },
];

/* ── Timeline ── */
function TimelineView() {
  const months = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
  const heights = [45,70,55,90,60,80,95,50,75,85,40,65];
  const colors = ["bg-violet-500","bg-blue-500","bg-violet-400","bg-blue-400","bg-violet-500","bg-cyan-500","bg-violet-400","bg-blue-500","bg-cyan-400","bg-violet-500","bg-blue-400","bg-cyan-500"];
  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 pt-3 pb-2">
        <div className="flex items-center gap-2">
          <span className="text-sm font-semibold text-zinc-200">Timeline</span>
          <span className="text-xs text-zinc-500">2023 – 2025</span>
        </div>
        <div className="flex gap-1">
          {["Year","Month","Week"].map((g,i)=>(
            <span key={g} className={`px-2 py-0.5 text-[10px] rounded border ${i===0?"bg-violet-600/30 text-violet-300 border-violet-500/20":"bg-zinc-800 text-zinc-400 border-zinc-700"}`}>{g}</span>
          ))}
        </div>
      </div>
      <div className="flex-1 flex items-end gap-[6px] px-4 pb-1 pt-2">
        {months.map((m,i)=>(
          <div key={m} className="flex-1 flex flex-col items-center gap-1">
            <div className={`w-full rounded-t ${colors[i]} opacity-80`} style={{height:`${heights[i]}%`,minHeight:8}}/>
            <span className="text-[9px] text-zinc-500">{m}</span>
          </div>
        ))}
      </div>
      <div className="flex items-center gap-4 px-4 py-2 border-t border-zinc-800 text-[10px] text-zinc-500">
        <span><strong className="text-zinc-300">1,247</strong> entries</span>
        <span><strong className="text-zinc-300">23</strong> themes</span>
        <span><strong className="text-zinc-300">156</strong> entities</span>
      </div>
    </div>
  );
}

/* ── Search ── */
function SearchView() {
  return (
    <div className="flex flex-col h-full">
      <div className="px-4 pt-3 pb-2">
        <span className="text-sm font-semibold text-zinc-200">Search</span>
        <div className="flex gap-2 mt-2">
          {["Hybrid","Keyword","Semantic"].map((m,i)=>(
            <span key={m} className={`px-2.5 py-1 text-[10px] rounded-md border ${i===0?"bg-violet-600/20 text-violet-300 border-violet-500/20":"bg-zinc-800 text-zinc-400 border-zinc-700"}`}>{m}</span>
          ))}
        </div>
      </div>
      <div className="px-4 py-2">
        <div className="flex items-center gap-2 bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2">
          <Search size={12} className="text-zinc-500"/>
          <span className="text-xs text-zinc-400">career growth and decisions</span>
        </div>
      </div>
      <div className="flex-1 px-4 space-y-2 overflow-hidden">
        {[
          {text:"I've been thinking about switching careers. The stability...", date:"2024-03-15", score:"0.92"},
          {text:"My mentor said that growth comes from discomfort. I agree...", date:"2024-06-22", score:"0.87"},
          {text:"Looking back at the decision to leave, it was the right...", date:"2024-11-03", score:"0.81"},
        ].map((r,i)=>(
          <div key={i} className="rounded-lg bg-zinc-800/50 border border-zinc-700/50 p-2.5">
            <p className="text-[11px] text-zinc-300 line-clamp-2">{r.text}</p>
            <div className="flex items-center gap-3 mt-1.5 text-[9px] text-zinc-500">
              <span>{r.date}</span><span>Score: {r.score}</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

/* ── Graph ── */
function GraphView() {
  const nodes = [
    {x:50,y:40,r:20,color:"#8b5cf6",label:"You"},
    {x:25,y:25,r:12,color:"#3b82f6",label:"Travel"},
    {x:78,y:28,r:14,color:"#06b6d4",label:"Work"},
    {x:35,y:68,r:11,color:"#f59e0b",label:"Music"},
    {x:72,y:65,r:13,color:"#10b981",label:"Friends"},
    {x:15,y:50,r:9,color:"#ec4899",label:"Books"},
    {x:88,y:50,r:10,color:"#6366f1",label:"Health"},
    {x:50,y:82,r:10,color:"#f97316",label:"Coding"},
    {x:60,y:15,r:8,color:"#14b8a6",label:"Ideas"},
  ];
  const edges = [[0,1],[0,2],[0,3],[0,4],[0,5],[0,6],[0,7],[1,8],[2,4],[2,6],[3,7],[4,7],[5,8]];
  return (
    <div className="relative h-full w-full">
      <svg viewBox="0 0 100 100" className="w-full h-full" preserveAspectRatio="xMidYMid meet">
        {edges.map(([a,b],i)=>(<line key={i} x1={nodes[a].x} y1={nodes[a].y} x2={nodes[b].x} y2={nodes[b].y} stroke="#3f3f46" strokeWidth="0.4" opacity="0.6"/>))}
        {nodes.map((n,i)=>(<g key={i}><circle cx={n.x} cy={n.y} r={n.r/2.5} fill={n.color} opacity="0.85"/><text x={n.x} y={n.y+n.r/2.5+4} textAnchor="middle" className="fill-zinc-400" fontSize="3">{n.label}</text></g>))}
      </svg>
      <div className="absolute top-3 right-3 w-28 bg-zinc-900/90 border border-zinc-700 rounded-lg p-2 text-[10px]">
        <div className="flex items-center gap-1 mb-1"><div className="w-2 h-2 rounded-full bg-violet-500"/><span className="text-zinc-200 font-medium">You</span></div>
        <div className="text-zinc-500 space-y-0.5"><div>Connections: 7</div><div>Mentions: 1,247</div></div>
      </div>
    </div>
  );
}

/* ── Ask/Chat ── */
function AskView() {
  return (
    <div className="flex h-full">
      <div className="w-32 border-r border-zinc-800 bg-zinc-900/30 p-2 space-y-1">
        <div className="px-2 py-1.5 rounded-md bg-violet-600/20 border border-violet-500/20 text-[10px] text-violet-300">New Chat</div>
        {["Career thoughts","Music taste","Travel plans"].map(t=>(
          <div key={t} className="px-2 py-1.5 rounded-md text-[10px] text-zinc-500 hover:bg-zinc-800 truncate">{t}</div>
        ))}
      </div>
      <div className="flex-1 flex flex-col">
        <div className="flex-1 overflow-hidden px-3 py-3 space-y-3">
          <div className="flex justify-end"><div className="max-w-[70%] rounded-2xl rounded-br-sm bg-violet-600/20 border border-violet-500/20 px-3 py-2 text-xs text-zinc-200">How has my interest in music changed?</div></div>
          <div className="flex justify-start"><div className="max-w-[80%] rounded-2xl rounded-bl-sm bg-zinc-800 border border-zinc-700 px-3 py-2 text-xs text-zinc-300 space-y-1.5">
            <p>Based on your entries, your music interests shifted:</p>
            <ul className="list-disc list-inside text-zinc-400 space-y-0.5 text-[11px]">
              <li><strong className="text-zinc-300">2023:</strong> Indie rock and lo-fi</li>
              <li><strong className="text-zinc-300">2024:</strong> Jazz and ambient</li>
              <li><strong className="text-zinc-300">Now:</strong> Classical and film scores</li>
            </ul>
            <div className="flex items-center gap-2 pt-1 border-t border-zinc-700 text-[10px] text-zinc-500"><span>23 sources</span><span className="w-px h-3 bg-zinc-700"/><span>High confidence</span></div>
          </div></div>
        </div>
        <div className="px-3 py-2 border-t border-zinc-800"><div className="flex items-center gap-2 bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-1.5"><span className="text-xs text-zinc-500 flex-1">Ask your memory...</span><div className="w-5 h-5 rounded bg-violet-600 flex items-center justify-center"><MessageCircle size={10} className="text-white"/></div></div></div>
      </div>
    </div>
  );
}

/* ── Insights ── */
function InsightsView() {
  return (
    <div className="flex flex-col h-full">
      <div className="px-4 pt-3 pb-2 flex items-center justify-between">
        <span className="text-sm font-semibold text-zinc-200">Insights</span>
        <span className="px-2.5 py-1 text-[10px] rounded-md bg-violet-600/20 text-violet-300 border border-violet-500/20">Run Analysis</span>
      </div>
      <div className="flex-1 px-4 space-y-2 overflow-hidden">
        {[
          {type:"Theme Shift", title:"Career focus intensified in Q3 2024", color:"text-blue-400", icon:TrendingUp},
          {type:"New Pattern", title:"Writing frequency doubled after moving", color:"text-emerald-400", icon:Activity},
          {type:"Belief Change", title:'Shifted from "stability first" to "growth first"', color:"text-amber-400", icon:Sparkles},
          {type:"Sentiment", title:"Overall positivity increased 23% year-over-year", color:"text-violet-400", icon:Zap},
          {type:"Entity", title:"Sarah mentioned 47 times across 3 platforms", color:"text-pink-400", icon:Brain},
        ].map((insight,i)=>(
          <div key={i} className="rounded-lg bg-zinc-800/50 border border-zinc-700/50 p-2.5 flex gap-2.5">
            <div className={`shrink-0 w-7 h-7 rounded-lg bg-zinc-800 flex items-center justify-center ${insight.color}`}><insight.icon size={13}/></div>
            <div>
              <span className={`text-[9px] font-medium ${insight.color}`}>{insight.type}</span>
              <p className="text-[11px] text-zinc-300 mt-0.5">{insight.title}</p>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

/* ── Evolution ── */
function EvolutionView() {
  return (
    <div className="flex flex-col h-full">
      <div className="px-4 pt-3 pb-2">
        <span className="text-sm font-semibold text-zinc-200">Evolution</span>
        <div className="flex gap-2 mt-2">
          {["Chart","Compare","Narratives"].map((t,i)=>(
            <span key={t} className={`px-2.5 py-1 text-[10px] rounded-md border ${i===0?"bg-violet-600/20 text-violet-300 border-violet-500/20":"bg-zinc-800 text-zinc-400 border-zinc-700"}`}>{t}</span>
          ))}
        </div>
      </div>
      <div className="flex-1 flex items-end gap-1 px-4 pb-2">
        {[30,45,35,60,50,70,65,55,80,75,45,90].map((h,i)=>(
          <div key={i} className="flex-1 flex gap-0.5">
            <div className="flex-1 rounded-t bg-violet-500/60" style={{height:`${h}%`}}/>
            <div className="flex-1 rounded-t bg-amber-500/60" style={{height:`${h*0.6}%`}}/>
          </div>
        ))}
      </div>
      <div className="flex items-center justify-center gap-4 px-4 py-2 border-t border-zinc-800 text-[10px] text-zinc-500">
        <span className="flex items-center gap-1"><div className="w-2 h-2 rounded bg-violet-500/60"/>Documents</span>
        <span className="flex items-center gap-1"><div className="w-2 h-2 rounded bg-amber-500/60"/>Extracted Facts</span>
      </div>
    </div>
  );
}

/* ── Import ── */
function ImportView() {
  const sources = [
    {name:"Google",color:"bg-red-500"},{name:"WhatsApp",color:"bg-green-500"},{name:"Twitter",color:"bg-sky-500"},{name:"Reddit",color:"bg-orange-500"},
    {name:"Obsidian",color:"bg-purple-500"},{name:"Notion",color:"bg-zinc-400"},{name:"Telegram",color:"bg-blue-500"},{name:"Discord",color:"bg-indigo-500"},
    {name:"Spotify",color:"bg-green-400"},{name:"YouTube",color:"bg-red-600"},{name:"Facebook",color:"bg-blue-600"},{name:"Instagram",color:"bg-pink-500"},
  ];
  return (
    <div className="flex flex-col h-full">
      <div className="px-4 pt-3 pb-2"><span className="text-sm font-semibold text-zinc-200">Import Sources</span><p className="text-[10px] text-zinc-500 mt-0.5">30+ platforms — drop any file or pick a source</p></div>
      <div className="mx-4 mb-3 border-2 border-dashed border-violet-500/30 rounded-lg py-4 flex flex-col items-center bg-violet-500/5"><Upload size={16} className="text-violet-400 mb-1"/><span className="text-[10px] text-zinc-400">Auto-detect & Import</span></div>
      <div className="flex-1 px-4 pb-3"><div className="grid grid-cols-4 gap-2">
        {sources.map(s=>(<div key={s.name} className="flex flex-col items-center gap-1 p-2 rounded-lg bg-zinc-800/50 border border-zinc-700/50"><div className={`w-5 h-5 rounded ${s.color} flex items-center justify-center text-[8px] font-bold text-white`}>{s.name[0]}</div><span className="text-[9px] text-zinc-400">{s.name}</span></div>))}
      </div></div>
    </div>
  );
}

/* ── Memory ── */
function MemoryView() {
  return (
    <div className="flex flex-col h-full">
      <div className="px-4 pt-3 pb-2 flex items-center justify-between">
        <span className="text-sm font-semibold text-zinc-200">Memory Browser</span>
        <div className="flex gap-1">
          {["All","Beliefs","Facts","Preferences"].map((c,i)=>(
            <span key={c} className={`px-2 py-0.5 text-[10px] rounded border ${i===0?"bg-violet-600/20 text-violet-300 border-violet-500/20":"bg-zinc-800 text-zinc-400 border-zinc-700"}`}>{c}</span>
          ))}
        </div>
      </div>
      <div className="flex-1 px-4 space-y-1.5 overflow-hidden">
        {[
          {text:"Values continuous learning over formal credentials", cat:"Belief", conf:92},
          {text:"Prefers remote work with occasional in-person meetups", cat:"Preference", conf:88},
          {text:"Started journaling consistently in March 2024", cat:"Fact", conf:95},
          {text:"Believes creativity thrives under constraints", cat:"Belief", conf:78},
          {text:"Moved to a new city for better opportunities", cat:"Fact", conf:91},
          {text:"Dislikes rigid hierarchical structures", cat:"Preference", conf:84},
        ].map((f,i)=>(
          <div key={i} className="rounded-lg bg-zinc-800/50 border border-zinc-700/50 px-3 py-2 flex items-start justify-between gap-2">
            <div>
              <p className="text-[11px] text-zinc-300">{f.text}</p>
              <div className="flex items-center gap-2 mt-1 text-[9px] text-zinc-500">
                <span className="text-violet-400">{f.cat}</span>
                <span>{f.conf}% confidence</span>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

/* ── Settings ── */
function SettingsView() {
  return (
    <div className="flex flex-col h-full overflow-hidden">
      <div className="px-4 pt-3 pb-2"><span className="text-sm font-semibold text-zinc-200">Settings</span></div>
      <div className="flex-1 px-4 space-y-3 overflow-hidden">
        <div className="rounded-lg bg-zinc-800/50 border border-zinc-700/50 p-3">
          <span className="text-xs font-medium text-zinc-300 flex items-center gap-1.5"><Zap size={12} className="text-violet-400"/>AI Provider</span>
          <div className="grid grid-cols-3 gap-1.5 mt-2">
            {["Gemini","Groq","Ollama"].map((p,i)=>(
              <div key={p} className={`px-2 py-1.5 rounded-md text-[10px] text-center border ${i===0?"bg-violet-600/20 text-violet-300 border-violet-500/20":"bg-zinc-800 text-zinc-400 border-zinc-700"}`}>
                {p}{i===0&&<span className="ml-1 text-[8px] text-emerald-400">FREE</span>}
              </div>
            ))}
          </div>
        </div>
        <div className="rounded-lg bg-zinc-800/50 border border-zinc-700/50 p-3">
          <span className="text-xs font-medium text-zinc-300 flex items-center gap-1.5"><Database size={12} className="text-violet-400"/>Data Overview</span>
          <div className="grid grid-cols-2 gap-2 mt-2">
            <div><span className="text-lg font-bold text-zinc-200">1,247</span><p className="text-[9px] text-zinc-500">Documents</p></div>
            <div><span className="text-lg font-bold text-zinc-200">89</span><p className="text-[9px] text-zinc-500">Memory Facts</p></div>
          </div>
        </div>
        <div className="rounded-lg bg-zinc-800/50 border border-zinc-700/50 p-3">
          <span className="text-xs font-medium text-zinc-300 flex items-center gap-1.5"><Shield size={12} className="text-emerald-400"/>Privacy</span>
          <div className="space-y-1 mt-1.5 text-[10px] text-zinc-400">
            <div className="flex items-center gap-1.5"><div className="w-1.5 h-1.5 rounded-full bg-emerald-400"/>All data stored locally</div>
            <div className="flex items-center gap-1.5"><div className="w-1.5 h-1.5 rounded-full bg-emerald-400"/>Zero telemetry</div>
            <div className="flex items-center gap-1.5"><div className="w-1.5 h-1.5 rounded-full bg-emerald-400"/>API keys in OS keychain</div>
          </div>
        </div>
      </div>
    </div>
  );
}

/* ── Main AppPreview ── */
export default function AppPreview() {
  const [activeView, setActiveView] = useState<ViewId>("timeline");

  const views: Record<ViewId, React.ReactNode> = {
    timeline: <TimelineView/>, search: <SearchView/>, ask: <AskView/>,
    insights: <InsightsView/>, evolution: <EvolutionView/>, import: <ImportView/>,
    memory: <MemoryView/>, graph: <GraphView/>, settings: <SettingsView/>,
  };

  return (
    <div className="rounded-xl overflow-hidden border border-zinc-700/50 shadow-2xl shadow-violet-500/5 bg-zinc-950">
      {/* macOS title bar */}
      <div className="flex items-center gap-2 px-4 py-3 bg-zinc-900 border-b border-zinc-800">
        <div className="flex items-center gap-1.5">
          <div className="w-3 h-3 rounded-full bg-red-500/80"/>
          <div className="w-3 h-3 rounded-full bg-yellow-500/80"/>
          <div className="w-3 h-3 rounded-full bg-green-500/80"/>
        </div>
        <div className="flex-1 text-center text-xs text-zinc-500 font-medium">MemryLab</div>
        <div className="w-12"/>
      </div>

      {/* Main area: sidebar + content */}
      <div className="flex" style={{height:400}}>
        {/* Sidebar */}
        <div className="w-14 bg-zinc-900/50 border-r border-zinc-800 flex flex-col items-center py-3 gap-0.5">
          {sidebarItems.map((item)=>(
            <button
              key={item.id}
              onClick={()=>setActiveView(item.id)}
              className={`w-10 h-10 rounded-lg flex flex-col items-center justify-center gap-0.5 transition-all ${
                activeView===item.id
                  ? "bg-violet-600/20 text-violet-400"
                  : "text-zinc-600 hover:text-zinc-400 hover:bg-zinc-800/50"
              }`}
              title={item.label}
            >
              <item.icon size={14}/>
              <span className="text-[7px] leading-none">{item.label}</span>
            </button>
          ))}
        </div>

        {/* Content */}
        <div className="flex-1 min-w-0">{views[activeView]}</div>
      </div>
    </div>
  );
}
