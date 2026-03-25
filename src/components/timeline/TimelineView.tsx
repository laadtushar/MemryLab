import { useEffect, useRef, useState } from "react";
import { commands, type TimelineDataResponse } from "@/lib/tauri";
import { BarChart3, ZoomIn, ZoomOut, RotateCcw } from "lucide-react";
import * as d3 from "d3";

export function TimelineView() {
  const [data, setData] = useState<TimelineDataResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const svgRef = useRef<SVGSVGElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [zoomLevel, setZoomLevel] = useState(1);

  useEffect(() => {
    commands
      .getTimelineData()
      .then(setData)
      .catch(() => setData(null))
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    if (!data || data.months.length === 0 || !svgRef.current || !containerRef.current) return;
    renderChart(data, svgRef.current, containerRef.current, zoomLevel);
  }, [data, zoomLevel]);

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground">
        Loading timeline...
      </div>
    );
  }

  if (!data || data.total_documents === 0) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center space-y-3">
          <BarChart3 size={48} className="mx-auto text-muted-foreground/50" />
          <h2 className="text-xl font-semibold">No data yet</h2>
          <p className="text-sm text-muted-foreground max-w-sm">
            Import your journals, notes, or documents to see your personal
            timeline here.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-border">
        <div>
          <h1 className="text-2xl font-semibold">Timeline</h1>
          <p className="text-sm text-muted-foreground">
            {data.total_documents} documents
            {data.date_range && (
              <span>
                {" "}&middot;{" "}
                {new Date(data.date_range.start).toLocaleDateString()} &mdash;{" "}
                {new Date(data.date_range.end).toLocaleDateString()}
              </span>
            )}
          </p>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={() => setZoomLevel((z) => Math.max(0.5, z - 0.25))}
            className="rounded-md p-2 hover:bg-secondary text-muted-foreground hover:text-foreground"
            title="Zoom out"
          >
            <ZoomOut size={16} />
          </button>
          <button
            onClick={() => setZoomLevel(1)}
            className="rounded-md p-2 hover:bg-secondary text-muted-foreground hover:text-foreground"
            title="Reset zoom"
          >
            <RotateCcw size={16} />
          </button>
          <button
            onClick={() => setZoomLevel((z) => Math.min(4, z + 0.25))}
            className="rounded-md p-2 hover:bg-secondary text-muted-foreground hover:text-foreground"
            title="Zoom in"
          >
            <ZoomIn size={16} />
          </button>
          <span className="text-xs text-muted-foreground ml-2 tabular-nums">
            {(zoomLevel * 100).toFixed(0)}%
          </span>
        </div>
      </div>

      {/* D3 Chart */}
      <div ref={containerRef} className="flex-1 overflow-x-auto overflow-y-hidden px-6 py-4">
        <svg ref={svgRef} className="w-full" />
      </div>

      {/* Monthly table */}
      <div className="border-t border-border max-h-48 overflow-y-auto">
        <table className="w-full text-sm">
          <thead className="sticky top-0 bg-background">
            <tr className="border-b border-border">
              <th className="text-left px-6 py-2 font-medium text-muted-foreground">Month</th>
              <th className="text-right px-6 py-2 font-medium text-muted-foreground">Documents</th>
              <th className="text-left px-6 py-2 font-medium text-muted-foreground">Activity</th>
            </tr>
          </thead>
          <tbody>
            {data.months.map((m) => {
              const maxCount = Math.max(...data.months.map((x) => x.document_count));
              const pct = maxCount > 0 ? (m.document_count / maxCount) * 100 : 0;
              return (
                <tr key={m.month} className="border-b border-border/30 last:border-0">
                  <td className="px-6 py-1.5 tabular-nums">{m.month}</td>
                  <td className="px-6 py-1.5 text-right tabular-nums">{m.document_count}</td>
                  <td className="px-6 py-1.5">
                    <div className="h-2 w-full max-w-48 rounded-full bg-secondary overflow-hidden">
                      <div
                        className="h-full bg-primary rounded-full"
                        style={{ width: `${pct}%` }}
                      />
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function renderChart(
  data: TimelineDataResponse,
  svg: SVGSVGElement,
  container: HTMLDivElement,
  zoom: number,
) {
  const months = data.months;
  if (months.length === 0) return;

  // Dimensions
  const containerWidth = container.clientWidth - 32;
  const width = Math.max(containerWidth, months.length * 40 * zoom);
  const height = 280;
  const margin = { top: 20, right: 20, bottom: 60, left: 50 };
  const innerWidth = width - margin.left - margin.right;
  const innerHeight = height - margin.top - margin.bottom;

  // Clear previous
  d3.select(svg).selectAll("*").remove();

  const root = d3
    .select(svg)
    .attr("width", width)
    .attr("height", height);

  const g = root
    .append("g")
    .attr("transform", `translate(${margin.left},${margin.top})`);

  // Scales
  const x = d3
    .scaleBand<string>()
    .domain(months.map((m) => m.month))
    .range([0, innerWidth])
    .padding(0.2);

  const maxVal = d3.max(months, (d) => d.document_count) ?? 1;
  const y = d3.scaleLinear().domain([0, maxVal]).nice().range([innerHeight, 0]);

  // Color scale based on value
  const color = d3
    .scaleSequential(d3.interpolateViridis)
    .domain([0, maxVal]);

  // Grid lines
  g.append("g")
    .attr("class", "grid")
    .selectAll("line")
    .data(y.ticks(5))
    .enter()
    .append("line")
    .attr("x1", 0)
    .attr("x2", innerWidth)
    .attr("y1", (d) => y(d))
    .attr("y2", (d) => y(d))
    .attr("stroke", "#27272a")
    .attr("stroke-dasharray", "2,4");

  // Bars
  const bars = g
    .selectAll(".bar")
    .data(months)
    .enter()
    .append("g")
    .attr("class", "bar");

  bars
    .append("rect")
    .attr("x", (d) => x(d.month)!)
    .attr("y", (d) => y(d.document_count))
    .attr("width", x.bandwidth())
    .attr("height", (d) => innerHeight - y(d.document_count))
    .attr("fill", (d) => color(d.document_count))
    .attr("rx", 3)
    .attr("opacity", 0.85)
    .on("mouseenter", function () {
      d3.select(this).attr("opacity", 1);
    })
    .on("mouseleave", function () {
      d3.select(this).attr("opacity", 0.85);
    });

  // Value labels on bars (only if bars are wide enough)
  if (x.bandwidth() > 25) {
    bars
      .append("text")
      .attr("x", (d) => x(d.month)! + x.bandwidth() / 2)
      .attr("y", (d) => y(d.document_count) - 4)
      .attr("text-anchor", "middle")
      .attr("fill", "#a1a1aa")
      .attr("font-size", "10px")
      .text((d) => (d.document_count > 0 ? d.document_count : ""));
  }

  // X axis
  const xAxis = g
    .append("g")
    .attr("transform", `translate(0,${innerHeight})`)
    .call(
      d3.axisBottom(x).tickSize(0).tickPadding(8),
    );

  xAxis.select(".domain").attr("stroke", "#27272a");
  xAxis
    .selectAll("text")
    .attr("fill", "#a1a1aa")
    .attr("font-size", "10px")
    .attr("transform", "rotate(-45)")
    .style("text-anchor", "end");

  // Show every Nth label if too many
  if (months.length > 20) {
    const step = Math.ceil(months.length / 20);
    xAxis.selectAll("text").each(function (_d, i) {
      if (i % step !== 0) d3.select(this).remove();
    });
  }

  // Y axis
  const yAxis = g.append("g").call(
    d3
      .axisLeft(y)
      .ticks(5)
      .tickSize(-4)
      .tickPadding(8),
  );

  yAxis.select(".domain").attr("stroke", "#27272a");
  yAxis.selectAll("text").attr("fill", "#a1a1aa").attr("font-size", "11px");
  yAxis.selectAll("line").attr("stroke", "#27272a");

  // Y axis label
  g.append("text")
    .attr("transform", "rotate(-90)")
    .attr("y", -margin.left + 14)
    .attr("x", -innerHeight / 2)
    .attr("text-anchor", "middle")
    .attr("fill", "#71717a")
    .attr("font-size", "11px")
    .text("Documents");
}
