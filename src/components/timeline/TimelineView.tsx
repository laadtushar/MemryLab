import { useEffect, useRef, useState, useCallback } from "react";
import {
  commands,
  type TimelineDataResponse,
  type TimelineBucket,
} from "@/lib/tauri";
import { BarChart3, ZoomIn, ZoomOut, RotateCcw } from "lucide-react";
import * as d3 from "d3";

type Granularity = "year" | "month" | "week" | "day";

const GRANULARITY_LABELS: Record<Granularity, string> = {
  year: "Yearly",
  month: "Monthly",
  week: "Weekly",
  day: "Daily",
};

/** Determine granularity from d3 zoom scale level */
function granularityForZoom(k: number): Granularity {
  if (k < 2) return "year";
  if (k < 12) return "month";
  if (k < 52) return "week";
  return "day";
}

/** Parse a period string to a Date */
function parsePeriod(period: string, granularity: Granularity): Date {
  switch (granularity) {
    case "year":
      return new Date(parseInt(period, 10), 0, 1);
    case "month": {
      const [y, m] = period.split("-").map(Number);
      return new Date(y, m - 1, 1);
    }
    case "week": {
      // Format: "YYYY-Www"
      const match = period.match(/^(\d{4})-W(\d{2})$/);
      if (match) {
        const year = parseInt(match[1], 10);
        const week = parseInt(match[2], 10);
        // Approximate: Jan 1 + week * 7 days
        const d = new Date(year, 0, 1 + week * 7);
        return d;
      }
      return new Date(period);
    }
    case "day": {
      const [y, m, d] = period.split("-").map(Number);
      return new Date(y, m - 1, d);
    }
  }
}

/** Get the width of one period bucket in milliseconds */
function periodDurationMs(granularity: Granularity): number {
  const day = 86400000;
  switch (granularity) {
    case "year":
      return day * 365;
    case "month":
      return day * 30;
    case "week":
      return day * 7;
    case "day":
      return day;
  }
}

export function TimelineView() {
  const [overview, setOverview] = useState<TimelineDataResponse | null>(null);
  const [data, setData] = useState<TimelineBucket[]>([]);
  const [granularity, setGranularity] = useState<Granularity>("month");
  const [loading, setLoading] = useState(true);
  const svgRef = useRef<SVGSVGElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const zoomRef = useRef<d3.ZoomBehavior<SVGSVGElement, unknown> | null>(null);
  const currentTransformRef = useRef<d3.ZoomTransform>(d3.zoomIdentity);
  const granularityRef = useRef<Granularity>("month");
  // Track if a fetch is in progress to avoid concurrent fetches
  const fetchingRef = useRef(false);

  // Load overview data (total docs, date range)
  useEffect(() => {
    commands
      .getTimelineData()
      .then((d) => {
        setOverview(d);
      })
      .catch(() => setOverview(null))
      .finally(() => setLoading(false));
  }, []);

  // Load granular data whenever granularity changes
  useEffect(() => {
    granularityRef.current = granularity;
    fetchingRef.current = true;
    commands
      .getDetailedTimeline(granularity)
      .then(setData)
      .catch(() => setData([]))
      .finally(() => {
        fetchingRef.current = false;
      });
  }, [granularity]);

  const renderChart = useCallback(() => {
    if (!svgRef.current || !containerRef.current || data.length === 0) return;

    const svg = svgRef.current;
    const container = containerRef.current;
    const g = granularity;

    renderZoomableChart(
      data,
      svg,
      container,
      g,
      zoomRef,
      currentTransformRef,
      (newG: Granularity) => {
        if (newG !== granularityRef.current && !fetchingRef.current) {
          setGranularity(newG);
        }
      },
    );
  }, [data, granularity]);

  useEffect(() => {
    renderChart();
  }, [renderChart]);

  // Re-render on window resize
  useEffect(() => {
    const handler = () => renderChart();
    window.addEventListener("resize", handler);
    return () => window.removeEventListener("resize", handler);
  }, [renderChart]);

  const handleZoomIn = useCallback(() => {
    if (!svgRef.current || !zoomRef.current) return;
    const svgSel = d3.select(svgRef.current);
    zoomRef.current.scaleBy(svgSel.transition().duration(300) as any, 2);
  }, []);

  const handleZoomOut = useCallback(() => {
    if (!svgRef.current || !zoomRef.current) return;
    const svgSel = d3.select(svgRef.current);
    zoomRef.current.scaleBy(svgSel.transition().duration(300) as any, 0.5);
  }, []);

  const handleZoomReset = useCallback(() => {
    if (!svgRef.current || !zoomRef.current) return;
    const svgSel = d3.select(svgRef.current);
    zoomRef.current.transform(
      svgSel.transition().duration(300) as any,
      d3.zoomIdentity,
    );
  }, []);

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground">
        Loading timeline...
      </div>
    );
  }

  if (!overview || overview.total_documents === 0) {
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
            {overview.total_documents} documents
            {overview.date_range && (
              <span>
                {" "}&middot;{" "}
                {new Date(overview.date_range.start).toLocaleDateString()}{" "}
                &mdash;{" "}
                {new Date(overview.date_range.end).toLocaleDateString()}
              </span>
            )}
          </p>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={handleZoomOut}
            className="rounded-md p-2 hover:bg-secondary text-muted-foreground hover:text-foreground"
            title="Zoom out"
          >
            <ZoomOut size={16} />
          </button>
          <button
            onClick={handleZoomReset}
            className="rounded-md p-2 hover:bg-secondary text-muted-foreground hover:text-foreground"
            title="Reset zoom"
          >
            <RotateCcw size={16} />
          </button>
          <button
            onClick={handleZoomIn}
            className="rounded-md p-2 hover:bg-secondary text-muted-foreground hover:text-foreground"
            title="Zoom in"
          >
            <ZoomIn size={16} />
          </button>
          <span className="ml-2 inline-flex items-center rounded-md bg-secondary px-2 py-0.5 text-xs font-medium text-secondary-foreground">
            {GRANULARITY_LABELS[granularity]}
          </span>
        </div>
      </div>

      {/* D3 Chart */}
      <div
        ref={containerRef}
        className="flex-1 overflow-hidden px-6 py-4"
      >
        <svg ref={svgRef} className="w-full" />
      </div>

      {/* Data table */}
      <div className="border-t border-border max-h-48 overflow-y-auto">
        <table className="w-full text-sm">
          <thead className="sticky top-0 bg-background">
            <tr className="border-b border-border">
              <th className="text-left px-6 py-2 font-medium text-muted-foreground">
                {GRANULARITY_LABELS[granularity]} Period
              </th>
              <th className="text-right px-6 py-2 font-medium text-muted-foreground">
                Documents
              </th>
              <th className="text-left px-6 py-2 font-medium text-muted-foreground">
                Activity
              </th>
            </tr>
          </thead>
          <tbody>
            {data.map((m) => {
              const maxCount = Math.max(...data.map((x) => x.count));
              const pct = maxCount > 0 ? (m.count / maxCount) * 100 : 0;
              return (
                <tr
                  key={m.period}
                  className="border-b border-border/30 last:border-0"
                >
                  <td className="px-6 py-1.5 tabular-nums">{m.period}</td>
                  <td className="px-6 py-1.5 text-right tabular-nums">
                    {m.count}
                  </td>
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

function renderZoomableChart(
  data: TimelineBucket[],
  svg: SVGSVGElement,
  container: HTMLDivElement,
  granularity: Granularity,
  zoomRef: React.MutableRefObject<d3.ZoomBehavior<SVGSVGElement, unknown> | null>,
  currentTransformRef: React.MutableRefObject<d3.ZoomTransform>,
  onGranularityChange: (g: Granularity) => void,
) {
  if (data.length === 0) return;

  // Dimensions
  const containerWidth = container.clientWidth - 32;
  const height = 320;
  const brushHeight = 30;
  const totalHeight = height + brushHeight + 10;
  const margin = { top: 20, right: 20, bottom: 40, left: 50 };
  const innerWidth = containerWidth - margin.left - margin.right;
  const innerHeight = height - margin.top - margin.bottom;

  // Parse dates
  const parsed = data.map((d) => ({
    date: parsePeriod(d.period, granularity),
    count: d.count,
    period: d.period,
  }));

  const durMs = periodDurationMs(granularity);

  // Time domain with padding
  const dateExtent = d3.extent(parsed, (d) => d.date) as [Date, Date];
  const domainStart = new Date(dateExtent[0].getTime() - durMs * 0.5);
  const domainEnd = new Date(
    dateExtent[1].getTime() + durMs * 1.5,
  );

  // Base scales
  const xBase = d3
    .scaleTime()
    .domain([domainStart, domainEnd])
    .range([0, innerWidth]);

  const maxVal = d3.max(parsed, (d) => d.count) ?? 1;
  const y = d3.scaleLinear().domain([0, maxVal]).nice().range([innerHeight, 0]);

  const color = d3
    .scaleSequential(d3.interpolateViridis)
    .domain([0, maxVal]);

  // Clear previous
  d3.select(svg).selectAll("*").remove();

  const root = d3
    .select(svg)
    .attr("width", containerWidth)
    .attr("height", totalHeight);

  // Clip path
  root
    .append("defs")
    .append("clipPath")
    .attr("id", "chart-clip")
    .append("rect")
    .attr("width", innerWidth)
    .attr("height", innerHeight);

  const g = root
    .append("g")
    .attr("transform", `translate(${margin.left},${margin.top})`);

  // Clipped area for bars
  const chartArea = g
    .append("g")
    .attr("clip-path", "url(#chart-clip)");

  // Grid lines
  const gridGroup = chartArea.append("g").attr("class", "grid");

  function drawGrid(yScale: d3.ScaleLinear<number, number>) {
    gridGroup.selectAll("*").remove();
    gridGroup
      .selectAll("line")
      .data(yScale.ticks(5))
      .enter()
      .append("line")
      .attr("x1", 0)
      .attr("x2", innerWidth)
      .attr("y1", (d) => yScale(d))
      .attr("y2", (d) => yScale(d))
      .attr("stroke", "#27272a")
      .attr("stroke-dasharray", "2,4");
  }
  drawGrid(y);

  // Bars group
  const barsGroup = chartArea.append("g").attr("class", "bars");

  function drawBars(xScale: d3.ScaleTime<number, number>) {
    barsGroup.selectAll("*").remove();

    const barWidth = Math.max(
      1,
      Math.abs(xScale(new Date(domainStart.getTime() + durMs)) - xScale(domainStart)) - 2,
    );

    barsGroup
      .selectAll("rect")
      .data(parsed)
      .enter()
      .append("rect")
      .attr("x", (d) => xScale(d.date))
      .attr("y", (d) => y(d.count))
      .attr("width", barWidth)
      .attr("height", (d) => innerHeight - y(d.count))
      .attr("fill", (d) => color(d.count))
      .attr("rx", Math.min(3, barWidth / 2))
      .attr("opacity", 0.85)
      .on("mouseenter", function () {
        d3.select(this).attr("opacity", 1);
      })
      .on("mouseleave", function () {
        d3.select(this).attr("opacity", 0.85);
      });

    // Value labels if bars are wide enough
    if (barWidth > 25) {
      barsGroup
        .selectAll("text")
        .data(parsed)
        .enter()
        .append("text")
        .attr("x", (d) => xScale(d.date) + barWidth / 2)
        .attr("y", (d) => y(d.count) - 4)
        .attr("text-anchor", "middle")
        .attr("fill", "#a1a1aa")
        .attr("font-size", "10px")
        .text((d) => (d.count > 0 ? d.count : ""));
    }
  }

  drawBars(xBase);

  // X axis
  const xAxisGroup = g
    .append("g")
    .attr("transform", `translate(0,${innerHeight})`)
    .call(d3.axisBottom(xBase).ticks(10).tickSize(0).tickPadding(8));

  xAxisGroup.select(".domain").attr("stroke", "#27272a");
  xAxisGroup
    .selectAll("text")
    .attr("fill", "#a1a1aa")
    .attr("font-size", "10px");

  // Y axis
  const yAxis = g.append("g").call(
    d3.axisLeft(y).ticks(5).tickSize(-4).tickPadding(8),
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

  // Brush area (mini chart at bottom)
  const brushMargin = { top: height + 5, left: margin.left };
  const brushG = root
    .append("g")
    .attr("transform", `translate(${brushMargin.left},${brushMargin.top})`);

  const yBrush = d3
    .scaleLinear()
    .domain([0, maxVal])
    .range([brushHeight, 0]);

  // Mini bars
  const miniBarWidth = Math.max(1, innerWidth / parsed.length - 1);
  brushG
    .selectAll("rect")
    .data(parsed)
    .enter()
    .append("rect")
    .attr("x", (d) => xBase(d.date))
    .attr("y", (d) => yBrush(d.count))
    .attr("width", miniBarWidth)
    .attr("height", (d) => brushHeight - yBrush(d.count))
    .attr("fill", "#3f3f46")
    .attr("opacity", 0.6);

  // Brush
  const brush = d3
    .brushX()
    .extent([
      [0, 0],
      [innerWidth, brushHeight],
    ])
    .on("end", (event: d3.D3BrushEvent<unknown>) => {
      if (!event.selection) return;
      const [x0, x1] = event.selection as [number, number];
      const newDomain = [xBase.invert(x0), xBase.invert(x1)] as [Date, Date];

      // Calculate scale factor
      const fullRange = domainEnd.getTime() - domainStart.getTime();
      const selectedRange = newDomain[1].getTime() - newDomain[0].getTime();
      const k = fullRange / selectedRange;
      const tx = -xBase(newDomain[0]) * k;

      const transform = d3.zoomIdentity.translate(tx, 0).scale(k);

      // Apply via zoom
      const svgSel = d3.select(svg);
      if (zoomRef.current) {
        svgSel.call(zoomRef.current.transform as any, transform);
      }

      // Clear brush selection
      brushG.call(brush.move as any, null);
    });

  brushG.append("g").call(brush);

  // Zoom behavior
  const zoom = d3
    .zoom<SVGSVGElement, unknown>()
    .scaleExtent([1, 365])
    .translateExtent([
      [0, 0],
      [innerWidth, height],
    ])
    .extent([
      [0, 0],
      [innerWidth, height],
    ])
    .on("zoom", (event: d3.D3ZoomEvent<SVGSVGElement, unknown>) => {
      const transform = event.transform;
      currentTransformRef.current = transform;

      const newX = transform.rescaleX(xBase);

      // Redraw bars with new scale
      drawBars(newX);

      // Update x axis
      xAxisGroup.call(
        d3.axisBottom(newX).ticks(10).tickSize(0).tickPadding(8) as any,
      );
      xAxisGroup.select(".domain").attr("stroke", "#27272a");
      xAxisGroup
        .selectAll("text")
        .attr("fill", "#a1a1aa")
        .attr("font-size", "10px");

      // Check if granularity should change
      const k = transform.k;
      const newGranularity = granularityForZoom(k);
      if (newGranularity !== granularity) {
        onGranularityChange(newGranularity);
      }
    });

  zoomRef.current = zoom;
  const svgSel = d3.select(svg);
  svgSel.call(zoom);

  // Restore current transform if we're re-rendering with new data
  if (currentTransformRef.current !== d3.zoomIdentity) {
    svgSel.call(zoom.transform, currentTransformRef.current);
  }
}
