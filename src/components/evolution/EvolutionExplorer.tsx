import { useEffect, useRef, useState } from "react";
import { commands, type EvolutionData } from "@/lib/tauri";
import { TrendingUp, Loader2 } from "lucide-react";
import * as d3 from "d3";

export function EvolutionExplorer() {
  const [data, setData] = useState<EvolutionData | null>(null);
  const [loading, setLoading] = useState(true);
  const svgRef = useRef<SVGSVGElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    commands
      .getEvolutionData()
      .then(setData)
      .catch(() => setData(null))
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    if (!data || data.months.length === 0 || !svgRef.current || !containerRef.current) return;
    renderDualChart(data, svgRef.current, containerRef.current);
  }, [data]);

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground">
        <Loader2 size={20} className="animate-spin mr-2" /> Loading...
      </div>
    );
  }

  if (!data || data.months.length === 0) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center space-y-3">
          <TrendingUp size={48} className="mx-auto text-muted-foreground/50" />
          <h2 className="text-xl font-semibold">No evolution data yet</h2>
          <p className="text-sm text-muted-foreground max-w-sm">
            Import documents and run analysis to see how your writing and
            extracted insights evolve over time.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <div className="border-b border-border px-6 py-4">
        <h1 className="text-2xl font-semibold flex items-center gap-2">
          <TrendingUp size={24} className="text-primary" /> Evolution
        </h1>
        <p className="text-sm text-muted-foreground mt-1">
          Track how your writing activity and extracted insights evolve over time.
          {data.date_range && (
            <span>
              {" "}{new Date(data.date_range[0]).toLocaleDateString()} —{" "}
              {new Date(data.date_range[1]).toLocaleDateString()}
            </span>
          )}
        </p>
      </div>

      <div ref={containerRef} className="flex-1 overflow-x-auto px-6 py-4">
        <svg ref={svgRef} className="w-full" />
      </div>

      {/* Legend */}
      <div className="border-t border-border px-6 py-3 flex items-center gap-6 text-xs text-muted-foreground">
        <div className="flex items-center gap-1.5">
          <div className="h-2.5 w-4 rounded-sm bg-primary/70" />
          Documents
        </div>
        <div className="flex items-center gap-1.5">
          <div className="h-2.5 w-4 rounded-sm bg-amber-400/70" />
          Extracted Facts
        </div>
        <div className="ml-auto tabular-nums">
          {data.months.reduce((s, m) => s + m.document_count, 0)} docs, {data.total_facts} facts
        </div>
      </div>
    </div>
  );
}

function renderDualChart(
  data: EvolutionData,
  svg: SVGSVGElement,
  container: HTMLDivElement,
) {
  const months = data.months;
  if (months.length === 0) return;

  const width = Math.max(container.clientWidth - 32, months.length * 50);
  const height = 320;
  const margin = { top: 20, right: 20, bottom: 60, left: 50 };
  const innerWidth = width - margin.left - margin.right;
  const innerHeight = height - margin.top - margin.bottom;

  d3.select(svg).selectAll("*").remove();

  const root = d3.select(svg).attr("width", width).attr("height", height);
  const g = root.append("g").attr("transform", `translate(${margin.left},${margin.top})`);

  const x = d3
    .scaleBand<string>()
    .domain(months.map((m) => m.month))
    .range([0, innerWidth])
    .padding(0.15);

  const maxDoc = d3.max(months, (d) => d.document_count) ?? 1;
  const maxFact = d3.max(months, (d) => d.fact_count) ?? 1;
  const maxVal = Math.max(maxDoc, maxFact);

  const y = d3.scaleLinear().domain([0, maxVal]).nice().range([innerHeight, 0]);

  // Grid
  g.selectAll(".grid-line")
    .data(y.ticks(5))
    .enter()
    .append("line")
    .attr("x1", 0)
    .attr("x2", innerWidth)
    .attr("y1", (d) => y(d))
    .attr("y2", (d) => y(d))
    .attr("stroke", "#27272a")
    .attr("stroke-dasharray", "2,4");

  const barWidth = x.bandwidth() / 2;

  // Document bars (left half)
  g.selectAll(".bar-doc")
    .data(months)
    .enter()
    .append("rect")
    .attr("x", (d) => x(d.month)!)
    .attr("y", (d) => y(d.document_count))
    .attr("width", barWidth - 1)
    .attr("height", (d) => innerHeight - y(d.document_count))
    .attr("fill", "#8b5cf6")
    .attr("opacity", 0.7)
    .attr("rx", 2);

  // Fact bars (right half)
  g.selectAll(".bar-fact")
    .data(months)
    .enter()
    .append("rect")
    .attr("x", (d) => x(d.month)! + barWidth + 1)
    .attr("y", (d) => y(d.fact_count))
    .attr("width", barWidth - 1)
    .attr("height", (d) => innerHeight - y(d.fact_count))
    .attr("fill", "#f59e0b")
    .attr("opacity", 0.7)
    .attr("rx", 2);

  // X axis
  const xAxis = g
    .append("g")
    .attr("transform", `translate(0,${innerHeight})`)
    .call(d3.axisBottom(x).tickSize(0).tickPadding(8));

  xAxis.select(".domain").attr("stroke", "#27272a");
  xAxis
    .selectAll("text")
    .attr("fill", "#a1a1aa")
    .attr("font-size", "10px")
    .attr("transform", "rotate(-45)")
    .style("text-anchor", "end");

  if (months.length > 20) {
    const step = Math.ceil(months.length / 20);
    xAxis.selectAll("text").each(function (_d, i) {
      if (i % step !== 0) d3.select(this).remove();
    });
  }

  // Y axis
  const yAxis = g.append("g").call(d3.axisLeft(y).ticks(5).tickSize(-4).tickPadding(8));
  yAxis.select(".domain").attr("stroke", "#27272a");
  yAxis.selectAll("text").attr("fill", "#a1a1aa").attr("font-size", "11px");
  yAxis.selectAll("line").attr("stroke", "#27272a");
}
