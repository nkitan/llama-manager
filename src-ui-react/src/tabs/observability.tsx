import { useMemo, useRef, useState } from "react";
import { Trash2, Activity, Server } from "lucide-react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useStore } from "@/store/app-store";
import { Button } from "@/components/ui/button";
import { cn, formatTime } from "@/lib/utils";

type Filter = "all" | "chat" | "agent" | "tools" | "errors";

const ROW_HEIGHT = 28;
const LOG_ROW_HEIGHT = 18;

export function ObservabilityTab() {
  const { obsEvents, clearObsEvents, serverLogs } = useStore();
  const [filter, setFilter] = useState<Filter>("all");
  const [showLogs, setShowLogs] = useState(false);

  const now = Date.now();

  const filtered = useMemo(() => {
    const events = [...obsEvents].reverse();
    if (filter === "all") return events;
    return events.filter((e) => {
      if (filter === "chat") return e.kind.startsWith("chat:");
      if (filter === "agent") return e.kind.startsWith("agent:") && !e.kind.includes("tool");
      if (filter === "tools") return e.kind.includes("tool");
      if (filter === "errors") return e.kind.includes("error");
      return true;
    });
  }, [obsEvents, filter]);

  const stats = useMemo(() => {
    const active = obsEvents.filter((e) => now - e.ts < 5000).length;
    const chatTokens = obsEvents.filter((e) => e.kind === "chat:token").length;
    const agentTokens = obsEvents.filter((e) => e.kind === "agent:token").length;
    const toolCalls = obsEvents.filter((e) => e.kind === "agent:tool_call").length;
    return { total: obsEvents.length, active, chatTokens, agentTokens, toolCalls };
  }, [obsEvents, now]);

  const FILTERS: { id: Filter; label: string }[] = [
    { id: "all", label: "All" },
    { id: "chat", label: "Chat" },
    { id: "agent", label: "Agent" },
    { id: "tools", label: "Tools" },
    { id: "errors", label: "Errors" },
  ];

  return (
    <div className="flex flex-col h-full overflow-hidden p-3 gap-3">
      {/* Header */}
      <div className="flex items-center justify-between flex-shrink-0">
        <div className="flex items-center gap-2">
          <Activity className="h-4 w-4 text-primary" />
          <span className="text-sm font-semibold">Observability</span>
          <div className="h-1.5 w-1.5 rounded-full bg-emerald-400 animate-pulse" />
        </div>
        <div className="flex gap-1">
          <Button variant="ghost" size="sm" onClick={() => setShowLogs(!showLogs)} className="text-xs h-7">
            {showLogs ? <><Activity className="h-3 w-3" /> Events</> : <><Server className="h-3 w-3" /> Server logs</>}
          </Button>
          <Button variant="ghost" size="icon-sm" onClick={clearObsEvents} title="Clear">
            <Trash2 className="h-3.5 w-3.5" />
          </Button>
        </div>
      </div>

      {/* Stats strip */}
      <div className="grid grid-cols-5 gap-2 flex-shrink-0">
        {[
          { label: "Total", value: stats.total },
          { label: "Active/5s", value: stats.active },
          { label: "Chat tokens", value: stats.chatTokens },
          { label: "Agent tokens", value: stats.agentTokens },
          { label: "Tool calls", value: stats.toolCalls },
        ].map((s) => (
          <div key={s.label} className="glass-card text-center py-2">
            <p className="text-base font-bold tabular-nums">{s.value.toLocaleString()}</p>
            <p className="text-[9px] text-muted-foreground">{s.label}</p>
          </div>
        ))}
      </div>

      {!showLogs ? (
        <>
          {/* Filter pills */}
          <div className="flex gap-1 flex-shrink-0">
            {FILTERS.map((f) => (
              <button
                key={f.id}
                onClick={() => setFilter(f.id)}
                className={cn(
                  "rounded-full px-2.5 py-0.5 text-[11px] font-medium transition-colors no-drag",
                  filter === f.id ? "bg-primary text-primary-foreground" : "bg-muted text-muted-foreground hover:text-foreground"
                )}
              >
                {f.label}
              </button>
            ))}
            <span className="ml-auto text-[10px] text-muted-foreground self-center">
              {filtered.length.toLocaleString()} events
            </span>
          </div>

          {/* Virtualized events list */}
          <VirtualizedEvents events={filtered} />
        </>
      ) : (
        <VirtualizedLogs logs={serverLogs} />
      )}
    </div>
  );
}

function VirtualizedEvents({ events }: { events: ReturnType<typeof useStore.getState>["obsEvents"] }) {
  const parentRef = useRef<HTMLDivElement>(null);
  const rowVirtualizer = useVirtualizer({
    count: events.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 20,
  });

  if (events.length === 0) {
    return (
      <div className="flex-1 glass-card rounded-lg flex items-center justify-center text-muted-foreground text-sm">
        No events. Trigger a chat or agent run.
      </div>
    );
  }

  return (
    <div ref={parentRef} className="flex-1 glass-card rounded-lg overflow-auto">
      <div style={{ height: rowVirtualizer.getTotalSize(), position: "relative" }}>
        {rowVirtualizer.getVirtualItems().map((vi) => {
          const ev = events[vi.index];
          const [bg, color] = ev.kind.startsWith("chat:")
            ? ["rgba(59,130,246,0.12)", "#3b82f6"]
            : ev.kind.includes("error")
            ? ["rgba(239,68,68,0.12)", "#ef4444"]
            : ev.kind.includes("tool")
            ? ["rgba(245,158,11,0.12)", "#f59e0b"]
            : ev.kind.startsWith("agent:")
            ? ["rgba(139,92,246,0.12)", "#8b5cf6"]
            : ["transparent", "hsl(var(--muted-foreground))"];
          return (
            <div
              key={vi.key}
              data-index={vi.index}
              ref={rowVirtualizer.measureElement}
              className="grid grid-cols-[112px_140px_80px_1fr] items-center gap-2 px-3 border-b border-border/20 hover:bg-muted/20 transition-colors text-[11px]"
              style={{ position: "absolute", top: 0, left: 0, right: 0, height: vi.size, transform: `translateY(${vi.start}px)` }}
            >
              <span className="font-mono text-muted-foreground/70 tabular-nums">{formatTime(ev.ts)}</span>
              <span className="text-[10px] px-1.5 py-0.5 rounded font-semibold justify-self-start" style={{ background: bg, color }}>{ev.kind}</span>
              <span className="font-mono text-muted-foreground truncate">{ev.id}</span>
              <span className="text-muted-foreground truncate">{ev.content.slice(0, 160)}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function VirtualizedLogs({ logs }: { logs: string[] }) {
  const parentRef = useRef<HTMLDivElement>(null);
  const rowVirtualizer = useVirtualizer({
    count: logs.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => LOG_ROW_HEIGHT,
    overscan: 50,
  });

  if (logs.length === 0) {
    return (
      <div className="flex-1 glass-card rounded-lg flex items-center justify-center text-muted-foreground text-sm p-3">
        No server logs yet.
      </div>
    );
  }

  // Display in reverse (newest first) without reversing the source.
  const indices = useMemo(() => Array.from({ length: logs.length }, (_, i) => logs.length - 1 - i), [logs.length]);

  return (
    <div ref={parentRef} className="flex-1 glass-card rounded-lg overflow-auto p-3 font-mono text-[10px]">
      <div style={{ height: rowVirtualizer.getTotalSize(), position: "relative" }}>
        {rowVirtualizer.getVirtualItems().map((vi) => {
          const line = logs[indices[vi.index]];
          return (
            <p
              key={vi.key}
              data-index={vi.index}
              ref={rowVirtualizer.measureElement}
              className={cn(
                "whitespace-pre-wrap break-all",
                line.includes("error") || line.includes("Error") ? "text-destructive" :
                line.includes("warn") || line.includes("WARN") ? "text-amber-400" :
                "text-muted-foreground"
              )}
              style={{ position: "absolute", top: 0, left: 0, right: 0, height: vi.size, transform: `translateY(${vi.start}px)` }}
            >
              {line}
            </p>
          );
        })}
      </div>
    </div>
  );
}
