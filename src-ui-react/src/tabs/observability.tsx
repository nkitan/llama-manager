import { useMemo, useState } from "react";
import { Trash2, Activity } from "lucide-react";
import { useStore } from "@/store/app-store";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { cn, formatTime } from "@/lib/utils";

type Filter = "all" | "chat" | "agent" | "tools" | "errors";

export function ObservabilityTab() {
  const { obsEvents, clearObsEvents, serverLogs } = useStore();
  const [filter, setFilter] = useState<Filter>("all");
  const [showLogs, setShowLogs] = useState(false);

  const now = Date.now();

  const filtered = useMemo(() => {
    const events = [...obsEvents].reverse();
    if (filter === "all") return events.slice(0, 200);
    return events
      .filter((e) => {
        if (filter === "chat") return e.kind.startsWith("chat:");
        if (filter === "agent") return e.kind.startsWith("agent:") && !e.kind.includes("tool");
        if (filter === "tools") return e.kind.includes("tool");
        if (filter === "errors") return e.kind.includes("error");
        return true;
      })
      .slice(0, 200);
  }, [obsEvents, filter]);

  const stats = useMemo(() => {
    const active = obsEvents.filter((e) => now - e.ts < 5000).length;
    const chatTokens = obsEvents.filter((e) => e.kind === "chat:token").length;
    const agentTokens = obsEvents.filter((e) => e.kind === "agent:token").length;
    const toolCalls = obsEvents.filter((e) => e.kind === "agent:tool_call").length;
    return { total: obsEvents.length, active, chatTokens, agentTokens, toolCalls };
  }, [obsEvents]);

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
            {showLogs ? "Events" : "Server logs"}
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

      {!showLogs && (
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
            <span className="ml-auto text-[10px] text-muted-foreground self-center">{filtered.length} events</span>
          </div>

          {/* Events table */}
          <ScrollArea className="flex-1 glass-card rounded-lg">
            <table className="w-full text-[11px]">
              <thead>
                <tr className="border-b border-border/40 sticky top-0" style={{ background: "hsl(var(--card) / 0.9)" }}>
                  <th className="text-left px-3 py-2 text-muted-foreground font-medium w-28">Time</th>
                  <th className="text-left px-2 py-2 text-muted-foreground font-medium w-36">Kind</th>
                  <th className="text-left px-2 py-2 text-muted-foreground font-medium w-24">ID</th>
                  <th className="text-left px-2 py-2 text-muted-foreground font-medium">Content</th>
                </tr>
              </thead>
              <tbody>
                {filtered.length === 0 ? (
                  <tr>
                    <td colSpan={4} className="text-center py-8 text-muted-foreground">No events</td>
                  </tr>
                ) : (
                  filtered.map((ev, i) => {
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
                      <tr key={i} className="border-b border-border/20 hover:bg-muted/20 transition-colors">
                        <td className="px-3 py-1.5 font-mono text-muted-foreground/70 tabular-nums">{formatTime(ev.ts)}</td>
                        <td className="px-2 py-1.5">
                          <span className="text-[10px] px-1.5 py-0.5 rounded font-semibold" style={{ background: bg, color }}>{ev.kind}</span>
                        </td>
                        <td className="px-2 py-1.5 font-mono text-muted-foreground truncate max-w-[80px]">{ev.id}</td>
                        <td className="px-2 py-1.5 text-muted-foreground truncate max-w-0">{ev.content.slice(0, 120)}</td>
                      </tr>
                    );
                  })
                )}
              </tbody>
            </table>
          </ScrollArea>
        </>
      )}

      {showLogs && (
        <ScrollArea className="flex-1 glass-card rounded-lg p-3">
          <div className="font-mono text-[10px] space-y-0.5">
            {serverLogs.length === 0 ? (
              <p className="text-muted-foreground">No server logs yet</p>
            ) : (
              [...serverLogs].reverse().map((line, i) => (
                <p key={i} className={cn(
                  "whitespace-pre-wrap break-all",
                  line.includes("error") || line.includes("Error") ? "text-destructive" :
                  line.includes("warn") || line.includes("WARN") ? "text-amber-400" :
                  "text-muted-foreground"
                )}>{line}</p>
              ))
            )}
          </div>
        </ScrollArea>
      )}
    </div>
  );
}
