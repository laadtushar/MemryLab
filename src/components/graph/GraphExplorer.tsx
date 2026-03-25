import { useEffect, useRef, useState, useCallback } from "react";
import { commands, type EntityGraphResponse, type EntityResponse } from "@/lib/tauri";
import * as d3 from "d3";
import { Loader2, Share2 } from "lucide-react";
import { NodeDetailPanel } from "./NodeDetailPanel";

const NODE_COLORS: Record<string, string> = {
  person: "#3b82f6",
  place: "#22c55e",
  organization: "#a855f7",
  concept: "#f59e0b",
  topic: "#f59e0b",
};
const DEFAULT_COLOR = "#71717a";

const TYPES = [
  { value: "", label: "All" },
  { value: "person", label: "People" },
  { value: "place", label: "Places" },
  { value: "organization", label: "Organizations" },
  { value: "concept", label: "Concepts" },
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
  const simulationRef = useRef<d3.Simulation<GraphNode, GraphEdge> | null>(null);

  const loadGraph = useCallback(async () => {
    setLoading(true);
    try {
      const data = await commands.getFullGraph(200, filter || undefined);
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

    svg.attr("width", width).attr("height", height);

    if (graphData.entities.length === 0) return;

    const nodes: GraphNode[] = graphData.entities.map((e) => ({
      id: e.id,
      name: e.name,
      entity_type: e.entity_type,
      mention_count: e.mention_count,
      first_seen: e.first_seen,
      last_seen: e.last_seen,
      radius: Math.max(6, Math.min(20, Math.sqrt(e.mention_count) * 3)),
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

    const showLabels = nodes.length < 100;

    // Container group for zoom/pan
    const g = svg.append("g");

    // Zoom behavior
    const zoom = d3
      .zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on("zoom", (event) => {
        g.attr("transform", event.transform);
      });
    svg.call(zoom);

    // Draw edges
    const link = g
      .append("g")
      .selectAll("line")
      .data(edges)
      .join("line")
      .attr("stroke", "#27272a")
      .attr("stroke-opacity", 0.3)
      .attr("stroke-width", 1);

    // Node groups
    const node = g
      .append("g")
      .selectAll<SVGGElement, GraphNode>("g")
      .data(nodes)
      .join("g")
      .style("cursor", "pointer")
      .on("click", (_event, d) => {
        setSelectedNode({
          id: d.id,
          name: d.name,
          entity_type: d.entity_type,
          mention_count: d.mention_count,
          first_seen: d.first_seen,
          last_seen: d.last_seen,
        });
      });

    // Circles
    node
      .append("circle")
      .attr("r", (d) => d.radius)
      .attr("fill", (d) => NODE_COLORS[d.entity_type] ?? DEFAULT_COLOR)
      .attr("stroke", (d) => {
        const color = NODE_COLORS[d.entity_type] ?? DEFAULT_COLOR;
        return d3.color(color)?.brighter(0.5)?.toString() ?? color;
      })
      .attr("stroke-width", 1.5);

    // Labels (always visible if < 100 nodes)
    if (showLabels) {
      node
        .append("text")
        .text((d) => d.name)
        .attr("x", (d) => d.radius + 4)
        .attr("y", 3)
        .attr("font-size", "10px")
        .attr("fill", "currentColor")
        .attr("class", "text-foreground")
        .style("pointer-events", "none");
    } else {
      // Tooltip on hover
      const tooltip = svg
        .append("g")
        .attr("class", "tooltip")
        .style("pointer-events", "none")
        .style("display", "none");

      const tooltipBg = tooltip
        .append("rect")
        .attr("rx", 4)
        .attr("ry", 4)
        .attr("fill", "#18181b")
        .attr("stroke", "#27272a")
        .attr("stroke-width", 1);

      const tooltipText = tooltip
        .append("text")
        .attr("fill", "#fafafa")
        .attr("font-size", "11px")
        .attr("text-anchor", "middle");

      node
        .on("mouseenter", (_event, d) => {
          tooltipText.text(d.name);
          const bbox = (tooltipText.node() as SVGTextElement).getBBox();
          tooltipBg
            .attr("x", bbox.x - 6)
            .attr("y", bbox.y - 3)
            .attr("width", bbox.width + 12)
            .attr("height", bbox.height + 6);
          tooltip
            .attr(
              "transform",
              `translate(${d.x ?? 0}, ${(d.y ?? 0) - d.radius - 10})`
            )
            .style("display", null);
        })
        .on("mouseleave", () => {
          tooltip.style("display", "none");
        });
    }

    // Drag behavior
    const drag = d3
      .drag<SVGGElement, GraphNode>()
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

    // Force simulation
    const simulation = d3
      .forceSimulation<GraphNode>(nodes)
      .force(
        "link",
        d3
          .forceLink<GraphNode, GraphEdge>(edges)
          .id((d) => d.id)
          .distance(80)
      )
      .force("charge", d3.forceManyBody().strength(-200))
      .force("center", d3.forceCenter(width / 2, height / 2))
      .force("collide", d3.forceCollide<GraphNode>().radius((d) => d.radius + 4))
      .on("tick", () => {
        link
          .attr("x1", (d) => (d.source as GraphNode).x ?? 0)
          .attr("y1", (d) => (d.source as GraphNode).y ?? 0)
          .attr("x2", (d) => (d.target as GraphNode).x ?? 0)
          .attr("y2", (d) => (d.target as GraphNode).y ?? 0);

        node.attr("transform", (d) => `translate(${d.x ?? 0},${d.y ?? 0})`);

        // Update tooltip position on hover for large graphs
        if (!showLabels) {
          svg.select(".tooltip").each(function () {
            // tooltip position is updated on mouseenter, no continuous update needed
          });
        }
      });

    simulationRef.current = simulation;

    return () => {
      simulation.stop();
    };
  }, [graphData]);

  return (
    <div className="flex h-full overflow-hidden">
      <div className="flex flex-1 flex-col overflow-hidden">
        {/* Header */}
        <div className="border-b border-border px-6 py-4">
          <div>
            <h1 className="text-2xl font-semibold flex items-center gap-2">
              <Share2 size={24} className="text-primary" /> Graph Explorer
            </h1>
            <p className="text-sm text-muted-foreground mt-1">
              Interactive force-directed graph of entities and their relationships.
            </p>
          </div>

          {/* Type filter */}
          <div className="flex gap-2 flex-wrap mt-3">
            {TYPES.map((t) => (
              <button
                key={t.value}
                onClick={() => setFilter(t.value)}
                className={`rounded-md px-3 py-1 text-sm transition-colors ${
                  filter === t.value
                    ? "bg-primary text-primary-foreground"
                    : "bg-secondary text-muted-foreground hover:text-foreground"
                }`}
              >
                {t.label}
              </button>
            ))}
          </div>
        </div>

        {/* Graph area */}
        <div ref={containerRef} className="flex-1 overflow-hidden relative">
          {loading ? (
            <div className="flex items-center justify-center h-full text-muted-foreground">
              <Loader2 size={20} className="animate-spin mr-2" /> Loading graph...
            </div>
          ) : !graphData || graphData.entities.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-center">
              <Share2 size={48} className="text-muted-foreground/30 mb-3" />
              <p className="text-lg font-medium text-muted-foreground">
                No entities yet
              </p>
              <p className="text-sm text-muted-foreground mt-1 max-w-sm">
                Import documents and run analysis to extract entities and build
                your knowledge graph.
              </p>
            </div>
          ) : (
            <svg ref={svgRef} className="w-full h-full" />
          )}
        </div>
      </div>

      {/* Detail panel */}
      {selectedNode && (
        <NodeDetailPanel
          entity={selectedNode}
          onClose={() => setSelectedNode(null)}
        />
      )}
    </div>
  );
}
