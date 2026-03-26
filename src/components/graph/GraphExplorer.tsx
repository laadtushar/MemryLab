import { useEffect, useRef, useState, useCallback } from "react";
import { commands, type EntityGraphResponse, type EntityResponse } from "@/lib/tauri";
import * as d3 from "d3";
import { Loader2, Share2, Maximize2, ZoomIn, ZoomOut, RotateCcw, User, MapPin, Building, Lightbulb } from "lucide-react";
import { NodeDetailPanel } from "./NodeDetailPanel";

const NODE_COLORS: Record<string, { fill: string; glow: string; dark_fill: string }> = {
  person: { fill: "#2563eb", glow: "#60a5fa", dark_fill: "#3b82f6" },
  place: { fill: "#16a34a", glow: "#4ade80", dark_fill: "#22c55e" },
  organization: { fill: "#9333ea", glow: "#c084fc", dark_fill: "#a855f7" },
  concept: { fill: "#d97706", glow: "#fbbf24", dark_fill: "#f59e0b" },
  topic: { fill: "#d97706", glow: "#fbbf24", dark_fill: "#f59e0b" },
};
const DEFAULT_COLORS = { fill: "#52525b", glow: "#a1a1aa", dark_fill: "#71717a" };

const TYPES = [
  { value: "", label: "All", icon: null },
  { value: "person", label: "People", icon: User },
  { value: "place", label: "Places", icon: MapPin },
  { value: "organization", label: "Orgs", icon: Building },
  { value: "concept", label: "Concepts", icon: Lightbulb },
];

interface GraphNode extends d3.SimulationNodeDatum {
  id: string;
  name: string;
  entity_type: string;
  mention_count: number;
  first_seen: string | null;
  last_seen: string | null;
  radius: number;
}

interface GraphEdge extends d3.SimulationLinkDatum<GraphNode> {
  id: string;
  rel_type: string;
  weight: number;
}

export function GraphExplorer() {
  const svgRef = useRef<SVGSVGElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState("");
  const [graphData, setGraphData] = useState<EntityGraphResponse | null>(null);
  const [selectedNode, setSelectedNode] = useState<EntityResponse | null>(null);
  const [hoveredNode, setHoveredNode] = useState<string | null>(null);
  const [nodeCount, setNodeCount] = useState(0);
  const [edgeCount, setEdgeCount] = useState(0);
  const simulationRef = useRef<d3.Simulation<GraphNode, GraphEdge> | null>(null);
  const zoomRef = useRef<d3.ZoomBehavior<SVGSVGElement, unknown> | null>(null);

  const loadGraph = useCallback(async () => {
    setLoading(true);
    try {
      const data = await commands.getFullGraph(300, filter || undefined);
      setGraphData(data);
    } catch {
      setGraphData(null);
    }
    setLoading(false);
  }, [filter]);

  useEffect(() => {
    loadGraph();
  }, [loadGraph]);

  useEffect(() => {
    if (!graphData || !svgRef.current || !containerRef.current) return;
    const svg = d3.select(svgRef.current);
    svg.selectAll("*").remove();

    const rect = containerRef.current.getBoundingClientRect();
    const width = rect.width;
    const height = rect.height;
    svg.attr("width", width).attr("height", height).attr("viewBox", `0 0 ${width} ${height}`);

    if (graphData.entities.length === 0) return;

    const isDark = !document.documentElement.classList.contains("light");
    const labelColor = isDark ? "#a1a1aa" : "#52525b";
    const labelShadow = isDark ? "0 1px 3px rgba(0,0,0,0.8)" : "0 1px 2px rgba(255,255,255,0.8)";
    const edgeLabelColor = isDark ? "#52525b" : "#a1a1aa";

    const nodes: GraphNode[] = graphData.entities.map((e) => ({
      id: e.id,
      name: e.name,
      entity_type: e.entity_type,
      mention_count: e.mention_count,
      first_seen: e.first_seen,
      last_seen: e.last_seen,
      radius: Math.max(8, Math.min(28, Math.sqrt(e.mention_count) * 4)),
    }));

    const nodeIdSet = new Set(nodes.map((n) => n.id));
    const edges: GraphEdge[] = graphData.relationships
      .filter((r) => nodeIdSet.has(r.source_entity_id) && nodeIdSet.has(r.target_entity_id))
      .map((r) => ({
        id: r.id,
        source: r.source_entity_id,
        target: r.target_entity_id,
        rel_type: r.rel_type,
        weight: r.weight,
      }));

    setNodeCount(nodes.length);
    setEdgeCount(edges.length);

    // ── SVG Defs: Glow filters + gradients ──
    const defs = svg.append("defs");

    // Glow filter for nodes
    Object.entries(NODE_COLORS).forEach(([type, colors]) => {
      const filter = defs.append("filter").attr("id", `glow-${type}`).attr("x", "-50%").attr("y", "-50%").attr("width", "200%").attr("height", "200%");
      filter.append("feGaussianBlur").attr("stdDeviation", "4").attr("result", "blur");
      filter.append("feFlood").attr("flood-color", colors.glow).attr("flood-opacity", isDark ? "0.6" : "0.3").attr("result", "color");
      filter.append("feComposite").attr("in", "color").attr("in2", "blur").attr("operator", "in").attr("result", "glow");
      const merge = filter.append("feMerge");
      merge.append("feMergeNode").attr("in", "glow");
      merge.append("feMergeNode").attr("in", "SourceGraphic");
    });

    // Default glow
    const defGlow = defs.append("filter").attr("id", "glow-default").attr("x", "-50%").attr("y", "-50%").attr("width", "200%").attr("height", "200%");
    defGlow.append("feGaussianBlur").attr("stdDeviation", "4").attr("result", "blur");
    defGlow.append("feFlood").attr("flood-color", DEFAULT_COLORS.glow).attr("flood-opacity", isDark ? "0.6" : "0.3").attr("result", "color");
    defGlow.append("feComposite").attr("in", "color").attr("in2", "blur").attr("operator", "in").attr("result", "glow");
    const defMerge = defGlow.append("feMerge");
    defMerge.append("feMergeNode").attr("in", "glow");
    defMerge.append("feMergeNode").attr("in", "SourceGraphic");

    // Highlight glow (stronger, for hovered/selected)
    const hiGlow = defs.append("filter").attr("id", "glow-highlight").attr("x", "-100%").attr("y", "-100%").attr("width", "300%").attr("height", "300%");
    hiGlow.append("feGaussianBlur").attr("stdDeviation", "8").attr("result", "blur");
    hiGlow.append("feFlood").attr("flood-color", "#ffffff").attr("flood-opacity", "0.4").attr("result", "color");
    hiGlow.append("feComposite").attr("in", "color").attr("in2", "blur").attr("operator", "in").attr("result", "glow");
    const hiMerge = hiGlow.append("feMerge");
    hiMerge.append("feMergeNode").attr("in", "glow");
    hiMerge.append("feMergeNode").attr("in", "SourceGraphic");

    // Edge gradient
    edges.forEach((e) => {
      const sourceNode = nodes.find((n) => n.id === (typeof e.source === "string" ? e.source : (e.source as GraphNode).id));
      const targetNode = nodes.find((n) => n.id === (typeof e.target === "string" ? e.target : (e.target as GraphNode).id));
      if (!sourceNode || !targetNode) return;
      const sourceColor = NODE_COLORS[sourceNode.entity_type]?.fill ?? DEFAULT_COLORS.fill;
      const targetColor = NODE_COLORS[targetNode.entity_type]?.fill ?? DEFAULT_COLORS.fill;
      const grad = defs.append("linearGradient").attr("id", `edge-${e.id}`).attr("gradientUnits", "userSpaceOnUse");
      grad.append("stop").attr("offset", "0%").attr("stop-color", sourceColor).attr("stop-opacity", 0.4);
      grad.append("stop").attr("offset", "100%").attr("stop-color", targetColor).attr("stop-opacity", 0.4);
    });

    // ── Container group ──
    const g = svg.append("g");

    // ── Background ──
    svg.insert("rect", ":first-child")
      .attr("width", width)
      .attr("height", height)
      .attr("fill", "transparent");

    // ── Zoom ──
    const zoom = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 6])
      .on("zoom", (event) => g.attr("transform", event.transform));
    svg.call(zoom);
    zoomRef.current = zoom;

    // ── Edges ──
    const link = g.append("g").attr("class", "edges")
      .selectAll("line")
      .data(edges)
      .join("line")
      .attr("stroke", (d) => `url(#edge-${d.id})`)
      .attr("stroke-width", (d) => Math.max(1, Math.min(3, d.weight)))
      .attr("stroke-opacity", isDark ? 0.4 : 0.6);

    // ── Edge labels (on hover) ──
    const edgeLabels = g.append("g").attr("class", "edge-labels")
      .selectAll("text")
      .data(edges)
      .join("text")
      .text((d) => d.rel_type)
      .attr("font-size", "8px")
      .attr("fill", edgeLabelColor)
      .attr("text-anchor", "middle")
      .attr("dy", -4)
      .style("opacity", 0)
      .style("pointer-events", "none");

    // ── Node groups ──
    const node = g.append("g").attr("class", "nodes")
      .selectAll<SVGGElement, GraphNode>("g")
      .data(nodes)
      .join("g")
      .style("cursor", "pointer");

    // Outer ring (pulse animation target)
    node.append("circle")
      .attr("class", "node-ring")
      .attr("r", (d) => d.radius + 3)
      .attr("fill", "none")
      .attr("stroke", (d) => NODE_COLORS[d.entity_type]?.glow ?? DEFAULT_COLORS.glow)
      .attr("stroke-width", 1)
      .attr("stroke-opacity", 0)
      .attr("stroke-dasharray", "3,3");

    // Main circle with glow
    node.append("circle")
      .attr("class", "node-circle")
      .attr("r", (d) => d.radius)
      .attr("fill", (d) => {
        const colors = NODE_COLORS[d.entity_type] ?? DEFAULT_COLORS;
        return isDark ? colors.dark_fill : colors.fill;
      })
      .attr("fill-opacity", isDark ? 0.85 : 0.95)
      .attr("stroke", (d) => NODE_COLORS[d.entity_type]?.glow ?? DEFAULT_COLORS.glow)
      .attr("stroke-width", 2)
      .attr("filter", (d) => `url(#glow-${NODE_COLORS[d.entity_type] ? d.entity_type : "default"})`);

    // Inner highlight dot (depth effect)
    node.append("circle")
      .attr("r", (d) => Math.max(2, d.radius * 0.3))
      .attr("cx", (d) => -d.radius * 0.15)
      .attr("cy", (d) => -d.radius * 0.15)
      .attr("fill", "white")
      .attr("fill-opacity", 0.2)
      .style("pointer-events", "none");

    // Labels
    node.append("text")
      .text((d) => d.name.length > 16 ? d.name.slice(0, 14) + "..." : d.name)
      .attr("x", (d) => d.radius + 6)
      .attr("y", 4)
      .attr("font-size", (d) => d.mention_count > 5 ? "11px" : "9px")
      .attr("font-weight", (d) => d.mention_count > 10 ? "600" : "400")
      .attr("fill", labelColor)
      .attr("opacity", (d) => d.mention_count > 2 ? 0.9 : 0.5)
      .style("pointer-events", "none")
      .style("text-shadow", labelShadow);

    // Entity type letter icon inside node (simple, always visible)
    const typeLetters: Record<string, string> = {
      person: "P",
      place: "L",
      organization: "O",
      concept: "C",
      topic: "T",
    };

    node.append("text")
      .text((d) => typeLetters[d.entity_type] ?? "?")
      .attr("text-anchor", "middle")
      .attr("dominant-baseline", "central")
      .attr("font-size", (d) => `${Math.max(10, d.radius * 0.9)}px`)
      .attr("font-weight", "700")
      .attr("font-family", "system-ui, sans-serif")
      .attr("fill", "white")
      .attr("fill-opacity", 0.95)
      .style("pointer-events", "none");

    // ── Interactions ──

    // Hover: highlight node + connected edges
    node.on("mouseenter", function (_event, d) {
      setHoveredNode(d.id);

      // Highlight + grow this node
      d3.select(this).select(".node-circle")
        .transition().duration(200)
        .attr("r", d.radius * 1.4)
        .attr("filter", "url(#glow-highlight)")
        .attr("fill-opacity", 1);

      d3.select(this).select(".node-ring")
        .transition().duration(200)
        .attr("r", d.radius * 1.4 + 3)
        .attr("stroke-opacity", 0.6);

      // Dim non-connected nodes
      const connectedIds = new Set<string>();
      connectedIds.add(d.id);
      edges.forEach((e) => {
        const sid = typeof e.source === "string" ? e.source : (e.source as GraphNode).id;
        const tid = typeof e.target === "string" ? e.target : (e.target as GraphNode).id;
        if (sid === d.id) connectedIds.add(tid);
        if (tid === d.id) connectedIds.add(sid);
      });

      node.transition().duration(200)
        .style("opacity", (n) => connectedIds.has(n.id) ? 1 : 0.15);

      // Highlight connected edges
      link.transition().duration(200)
        .attr("stroke-opacity", (e) => {
          const sid = typeof e.source === "string" ? e.source : (e.source as GraphNode).id;
          const tid = typeof e.target === "string" ? e.target : (e.target as GraphNode).id;
          return sid === d.id || tid === d.id ? 0.8 : 0.05;
        })
        .attr("stroke-width", (e) => {
          const sid = typeof e.source === "string" ? e.source : (e.source as GraphNode).id;
          const tid = typeof e.target === "string" ? e.target : (e.target as GraphNode).id;
          return sid === d.id || tid === d.id ? 2.5 : 1;
        });

      // Show edge labels for connected edges
      edgeLabels.transition().duration(200)
        .style("opacity", (e) => {
          const sid = typeof e.source === "string" ? e.source : (e.source as GraphNode).id;
          const tid = typeof e.target === "string" ? e.target : (e.target as GraphNode).id;
          return sid === d.id || tid === d.id ? 1 : 0;
        });
    });

    node.on("mouseleave", function () {
      setHoveredNode(null);

      // Restore all nodes
      node.transition().duration(300).style("opacity", 1);
      node.selectAll(".node-circle")
        .transition().duration(300)
        .attr("r", function () {
          const d = d3.select(this.parentNode as Element).datum() as GraphNode;
          return d.radius;
        })
        .attr("filter", function () {
          const d = d3.select(this.parentNode as Element).datum() as GraphNode;
          return `url(#glow-${NODE_COLORS[d.entity_type] ? d.entity_type : "default"})`;
        })
        .attr("fill-opacity", 0.85);
      node.selectAll(".node-ring")
        .transition().duration(300)
        .attr("r", function () {
          const d = d3.select(this.parentNode as Element).datum() as GraphNode;
          return d.radius + 3;
        })
        .attr("stroke-opacity", 0);

      // Restore edges
      link.transition().duration(300)
        .attr("stroke-opacity", 0.4)
        .attr("stroke-width", (d) => Math.max(1, Math.min(3, d.weight)));

      edgeLabels.transition().duration(300).style("opacity", 0);
    });

    // Click: select node
    node.on("click", (_event, d) => {
      setSelectedNode({
        id: d.id,
        name: d.name,
        entity_type: d.entity_type,
        mention_count: d.mention_count,
        first_seen: d.first_seen,
        last_seen: d.last_seen,
      });
    });

    // ── Drag ──
    const drag = d3.drag<SVGGElement, GraphNode>()
      .on("start", (event, d) => {
        if (!event.active) simulation.alphaTarget(0.3).restart();
        d.fx = d.x;
        d.fy = d.y;
      })
      .on("drag", (event, d) => {
        d.fx = event.x;
        d.fy = event.y;
      })
      .on("end", (event, d) => {
        if (!event.active) simulation.alphaTarget(0);
        d.fx = null;
        d.fy = null;
      });
    node.call(drag);

    // ── Force simulation ──
    const simulation = d3.forceSimulation<GraphNode>(nodes)
      .force("link", d3.forceLink<GraphNode, GraphEdge>(edges).id((d) => d.id).distance(100).strength(0.5))
      .force("charge", d3.forceManyBody().strength(-250).distanceMax(400))
      .force("center", d3.forceCenter(width / 2, height / 2).strength(0.05))
      .force("collide", d3.forceCollide<GraphNode>().radius((d) => d.radius + 8).strength(0.7))
      .force("x", d3.forceX(width / 2).strength(0.02))
      .force("y", d3.forceY(height / 2).strength(0.02))
      .alphaDecay(0.02)
      .velocityDecay(0.3)
      .on("tick", () => {
        link
          .attr("x1", (d) => (d.source as GraphNode).x ?? 0)
          .attr("y1", (d) => (d.source as GraphNode).y ?? 0)
          .attr("x2", (d) => (d.target as GraphNode).x ?? 0)
          .attr("y2", (d) => (d.target as GraphNode).y ?? 0);

        edgeLabels
          .attr("x", (d) => (((d.source as GraphNode).x ?? 0) + ((d.target as GraphNode).x ?? 0)) / 2)
          .attr("y", (d) => (((d.source as GraphNode).y ?? 0) + ((d.target as GraphNode).y ?? 0)) / 2);

        node.attr("transform", (d) => `translate(${d.x ?? 0},${d.y ?? 0})`);
      });

    simulationRef.current = simulation;

    // Gentle initial animation — start nodes from center
    nodes.forEach((n) => {
      n.x = width / 2 + (Math.random() - 0.5) * 100;
      n.y = height / 2 + (Math.random() - 0.5) * 100;
    });
    simulation.alpha(1).restart();

    return () => simulation.stop();
  }, [graphData]);

  const handleZoomIn = () => {
    if (svgRef.current && zoomRef.current) {
      d3.select(svgRef.current).transition().duration(300).call(zoomRef.current.scaleBy as any, 1.5);
    }
  };
  const handleZoomOut = () => {
    if (svgRef.current && zoomRef.current) {
      d3.select(svgRef.current).transition().duration(300).call(zoomRef.current.scaleBy as any, 0.67);
    }
  };
  const handleZoomReset = () => {
    if (svgRef.current && zoomRef.current) {
      d3.select(svgRef.current).transition().duration(500).call(zoomRef.current.transform as any, d3.zoomIdentity);
    }
  };
  const handleFitView = () => {
    if (simulationRef.current) {
      simulationRef.current.alpha(0.5).restart();
    }
    handleZoomReset();
  };

  return (
    <div className="flex h-full overflow-hidden">
      <div className="flex flex-1 flex-col overflow-hidden">
        {/* Header */}
        <div className="border-b border-border px-6 py-4">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-semibold flex items-center gap-2">
                <Share2 size={24} className="text-primary" /> Graph Explorer
              </h1>
              {nodeCount > 0 && (
                <p className="text-sm text-muted-foreground mt-0.5">
                  {nodeCount} entities, {edgeCount} relationships
                  {hoveredNode && <span className="text-primary"> — hover to explore connections</span>}
                </p>
              )}
            </div>

            {/* Zoom controls */}
            {nodeCount > 0 && (
              <div className="flex items-center gap-1">
                <button onClick={handleZoomOut} className="rounded-md p-1.5 hover:bg-secondary text-muted-foreground hover:text-foreground transition-colors" title="Zoom out">
                  <ZoomOut size={16} />
                </button>
                <button onClick={handleZoomReset} className="rounded-md p-1.5 hover:bg-secondary text-muted-foreground hover:text-foreground transition-colors" title="Reset">
                  <RotateCcw size={16} />
                </button>
                <button onClick={handleZoomIn} className="rounded-md p-1.5 hover:bg-secondary text-muted-foreground hover:text-foreground transition-colors" title="Zoom in">
                  <ZoomIn size={16} />
                </button>
                <button onClick={handleFitView} className="rounded-md p-1.5 hover:bg-secondary text-muted-foreground hover:text-foreground transition-colors" title="Re-simulate">
                  <Maximize2 size={16} />
                </button>
              </div>
            )}
          </div>

          {/* Type filters */}
          <div className="flex gap-1.5 flex-wrap mt-3">
            {TYPES.map((t) => {
              const Icon = t.icon;
              return (
                <button
                  key={t.value}
                  onClick={() => setFilter(t.value)}
                  className={`rounded-full px-3 py-1 text-xs font-medium transition-all duration-200 flex items-center gap-1.5 ${
                    filter === t.value
                      ? "bg-primary text-primary-foreground shadow-lg shadow-primary/25"
                      : "bg-secondary/50 text-muted-foreground hover:bg-secondary hover:text-foreground"
                  }`}
                >
                  {Icon && <Icon size={12} style={{ color: filter === t.value ? undefined : NODE_COLORS[t.value]?.fill }} />}
                  {t.label}
                </button>
              );
            })}
          </div>
        </div>

        {/* Graph area */}
        <div ref={containerRef} className="flex-1 overflow-hidden relative bg-[radial-gradient(circle_at_center,_rgba(59,130,246,0.04)_0%,_transparent_70%)]">
          {loading ? (
            <div className="flex items-center justify-center h-full text-muted-foreground">
              <Loader2 size={20} className="animate-spin mr-2" /> Loading graph...
            </div>
          ) : !graphData || graphData.entities.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-center">
              <Share2 size={48} className="text-muted-foreground/20 mb-3" />
              <p className="text-lg font-medium text-muted-foreground">No entities yet</p>
              <p className="text-sm text-muted-foreground mt-1 max-w-sm">
                Import documents and run analysis to extract entities and build your knowledge graph.
              </p>
            </div>
          ) : (
            <svg ref={svgRef} className="w-full h-full" />
          )}

          {/* Legend overlay */}
          {nodeCount > 0 && (
            <div className="absolute bottom-4 left-4 flex gap-3 rounded-lg bg-background/70 backdrop-blur-sm border border-border/50 px-3 py-2">
              {TYPES.filter((t) => t.value).map((t) => {
                const Icon = t.icon;
                const colors = NODE_COLORS[t.value] ?? DEFAULT_COLORS;
                return (
                  <div key={t.value} className="flex items-center gap-1.5 text-[10px] text-muted-foreground">
                    {Icon && <Icon size={10} style={{ color: colors.fill }} />}
                    {t.label}
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>

      {/* Detail panel */}
      {selectedNode && (
        <NodeDetailPanel entity={selectedNode} onClose={() => setSelectedNode(null)} />
      )}
    </div>
  );
}
