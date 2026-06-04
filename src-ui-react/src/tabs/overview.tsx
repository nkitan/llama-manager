import { useMemo } from "react";
import { useStore } from "@/store/app-store";

interface NodeDef {
  id: string;
  label: string;
  icon: string;
  x: number;
  y: number;
  color: string;
}

interface EdgeDef {
  from: string;
  to: string;
}

const NODES: NodeDef[] = [
  { id: "llama-server",  label: "llama-server",  icon: "🦙", x: 440, y: 60,  color: "#6366f1" },
  { id: "chat",          label: "Chat",           icon: "💬", x: 170, y: 170, color: "#3b82f6" },
  { id: "agent-engine",  label: "Agent Engine",   icon: "🧠", x: 710, y: 170, color: "#8b5cf6" },
  { id: "mcp-tools",     label: "MCP Tools",      icon: "🔌", x: 540, y: 290, color: "#14b8a6" },
  { id: "web-search",    label: "Web Search",     icon: "🌐", x: 670, y: 290, color: "#0ea5e9" },
  { id: "file-tools",    label: "File Tools",     icon: "📄", x: 800, y: 290, color: "#f59e0b" },
  { id: "shell",         label: "Shell",          icon: "⚡", x: 410, y: 290, color: "#ef4444" },
  { id: "memory",        label: "Memory",         icon: "💾", x: 100, y: 410, color: "#10b981" },
  { id: "notes",         label: "Quick Notes",    icon: "📝", x: 240, y: 410, color: "#84cc16" },
  { id: "todos",         label: "Todos",          icon: "✓",  x: 370, y: 410, color: "#22c55e" },
  { id: "calendar",      label: "Calendar",       icon: "📅", x: 500, y: 410, color: "#f97316" },
  { id: "planner",       label: "Task Planner",   icon: "🗂", x: 630, y: 410, color: "#ec4899" },
  { id: "deep-research", label: "Deep Research",  icon: "🔬", x: 760, y: 500, color: "#a855f7" },
];

const EDGES: EdgeDef[] = [
  { from: "llama-server", to: "chat" },
  { from: "llama-server", to: "agent-engine" },
  { from: "chat",         to: "agent-engine" },
  { from: "agent-engine", to: "mcp-tools" },
  { from: "agent-engine", to: "web-search" },
  { from: "agent-engine", to: "file-tools" },
  { from: "agent-engine", to: "shell" },
  { from: "agent-engine", to: "memory" },
  { from: "agent-engine", to: "notes" },
  { from: "agent-engine", to: "todos" },
  { from: "agent-engine", to: "calendar" },
  { from: "agent-engine", to: "planner" },
  { from: "agent-engine", to: "deep-research" },
  { from: "memory",       to: "chat" },
];

const posMap = new Map(NODES.map((n) => [n.id, { x: n.x, y: n.y }]));

function classifyEventNodes(kind: string, id: string): string[] {
  const result: string[] = [];
  if (kind.startsWith("chat:")) { result.push("chat", "llama-server"); }
  if (kind.startsWith("agent:")) { result.push("agent-engine"); }
  if (kind === "agent:tool_call" || kind === "agent:tool_ok" || kind === "agent:tool_err") {
    const l = id.toLowerCase();
    if (l.includes("web") || l.includes("search") || l.includes("searx")) result.push("web-search");
    if (l.includes("file") || l.includes("read") || l.includes("write")) result.push("file-tools");
    if (l.includes("mcp"))      result.push("mcp-tools");
    if (l.includes("note"))     result.push("notes");
    if (l.includes("todo"))     result.push("todos");
    if (l.includes("calendar")) result.push("calendar");
    if (l.includes("planner") || l.includes("task")) result.push("planner");
    if (l.includes("memory") || l.includes("obsidian")) result.push("memory");
    if (l.includes("run_command") || l.includes("shell") || l.includes("bash")) result.push("shell");
    if (l.includes("research")) result.push("deep-research");
  }
  return result;
}

export function OverviewTab() {
  const obsEvents = useStore((s) => s.obsEvents);

  const activeSet = useMemo(() => {
    const now = Date.now();
    const active = new Set<string>();
    for (let i = obsEvents.length - 1; i >= 0; i--) {
      const ev = obsEvents[i];
      if (now - ev.ts < 4000) {
        classifyEventNodes(ev.kind, ev.id).forEach((n) => active.add(n));
      } else break;
    }
    return active;
  }, [obsEvents]);

  const recentEvents = useMemo(() => {
    const now = Date.now();
    return [...obsEvents].reverse().filter((e) => now - e.ts < 30000).slice(0, 14);
  }, [obsEvents]);

  return (
    <div className="flex flex-col h-full overflow-hidden p-3 gap-3">
      {/* Header */}
      <div className="flex items-center justify-between flex-shrink-0">
        <div>
          <h2 className="text-sm font-semibold">System Overview</h2>
          <p className="text-[10px] text-muted-foreground">Live wiring — nodes glow when active</p>
        </div>
        <div className="flex items-center gap-2">
          <div
            className="h-2 w-2 rounded-full transition-colors"
            style={{
              background: activeSet.size > 0 ? "#10b981" : "hsl(var(--muted-foreground))",
              boxShadow: activeSet.size > 0 ? "0 0 0 3px rgba(16,185,129,0.25)" : "none",
            }}
          />
          <span className="text-xs text-muted-foreground">
            {activeSet.size > 0 ? `${activeSet.size} active` : "Idle"}
          </span>
        </div>
      </div>

      <div className="flex flex-1 min-h-0 gap-3">
        {/* SVG diagram */}
        <div className="glass-card flex-1 flex items-center justify-center overflow-hidden">
          <svg
            viewBox="0 0 920 560"
            width="100%"
            height="100%"
            style={{ maxHeight: "calc(100vh - 240px)" }}
          >
            <defs>
              <marker id="arrow" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto">
                <path d="M0,0 L0,6 L8,3 z" fill="hsl(var(--border))" />
              </marker>
              <marker id="arrow-active" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto">
                <path d="M0,0 L0,6 L8,3 z" fill="#10b981" />
              </marker>
            </defs>

            {/* Edges */}
            {EDGES.map((edge, i) => {
              const from = posMap.get(edge.from);
              const to = posMap.get(edge.to);
              if (!from || !to) return null;
              const isActive = activeSet.has(edge.from) && activeSet.has(edge.to);
              return (
                <line
                  key={i}
                  x1={from.x} y1={from.y} x2={to.x} y2={to.y}
                  stroke={isActive ? "#10b981" : "hsl(var(--border))"}
                  strokeWidth={isActive ? 2.5 : 1.5}
                  strokeOpacity={0.7}
                  markerEnd={isActive ? "url(#arrow-active)" : "url(#arrow)"}
                />
              );
            })}

            {/* Nodes */}
            {NODES.map((node) => {
              const isActive = activeSet.has(node.id);
              return (
                <g key={node.id} transform={`translate(${node.x},${node.y})`}>
                  {isActive && (
                    <ellipse cx="0" cy="0" rx="46" ry="28" fill="none" stroke={node.color} strokeWidth="3" strokeOpacity="0.5">
                      <animate attributeName="stroke-opacity" values="0.5;0.9;0.5" dur="1.2s" repeatCount="indefinite" />
                    </ellipse>
                  )}
                  <rect
                    x="-42" y="-20" width="84" height="40" rx="10"
                    fill={isActive ? `${node.color}22` : "hsl(var(--card))"}
                    stroke={node.color}
                    strokeWidth={isActive ? 2 : 1}
                    strokeOpacity={0.8}
                  />
                  <text x="-22" y="5" fontSize="13" textAnchor="middle" dominantBaseline="middle">
                    {node.icon}
                  </text>
                  <text x="10" y="0" fontSize="9.5" fill="hsl(var(--foreground))" textAnchor="middle" dominantBaseline="middle" fontWeight="600">
                    {node.label}
                  </text>
                </g>
              );
            })}
          </svg>
        </div>

        {/* Recent events feed */}
        <div className="glass-card w-56 flex-shrink-0 flex flex-col overflow-hidden">
          <div className="px-3 py-2 border-b border-border/40 text-[11px] font-semibold">Recent Activity</div>
          <div className="flex-1 overflow-y-auto p-1.5 space-y-1">
            {recentEvents.length === 0 ? (
              <p className="text-center text-xs text-muted-foreground mt-4">No recent events</p>
            ) : (
              recentEvents.map((ev, i) => {
                const [bg, color] = ev.kind.startsWith("chat:")
                  ? ["rgba(59,130,246,0.12)", "#3b82f6"]
                  : ev.kind.includes("tool")
                  ? ["rgba(245,158,11,0.12)", "#f59e0b"]
                  : ev.kind.startsWith("agent:")
                  ? ["rgba(139,92,246,0.12)", "#8b5cf6"]
                  : ["rgba(255,255,255,0.05)", "hsl(var(--muted-foreground))"];
                return (
                  <div key={i} className="glass-card p-1.5 space-y-0.5">
                    <div className="flex items-center gap-1.5">
                      <span className="text-[9px] px-1.5 py-0.5 rounded font-semibold" style={{ background: bg, color }}>{ev.kind}</span>
                      {ev.id && <span className="text-[9px] text-muted-foreground truncate max-w-[80px]">{ev.id}</span>}
                    </div>
                    {ev.content && (
                      <p className="text-[9px] text-muted-foreground truncate">{ev.content.slice(0, 40)}</p>
                    )}
                  </div>
                );
              })
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
