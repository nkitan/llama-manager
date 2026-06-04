import { useEffect, useRef, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { Bot, Send, Trash2, AlertTriangle, Activity } from "lucide-react";
import { ipc, type MonitorState } from "@/lib/ipc";
import { useStore } from "@/store/app-store";
import { resolveTarget } from "@/lib/target";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

export function MonitorTab() {
  const [state, setState] = useState<MonitorState>({ agents: [], events: [] });
  const [task, setTask] = useState("");
  const [dispatching, setDispatching] = useState(false);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const config = useStore((s) => s.config);

  // Reverse the events array so newest is first; memoize it
  const reversedEvents = [...state.events].reverse();

  const load = async () => {
    try {
      const s = await ipc.monitorLoad();
      setState(s);
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    load();
    pollRef.current = setInterval(load, 2000);
    return () => {
      if (pollRef.current) clearInterval(pollRef.current);
    };
  }, []);

  const clearLogs = async () => {
    try {
      const s = await ipc.monitorClearEvents();
      setState(s);
    } catch (e) {
      console.error(e);
    }
  };

  const dispatch = async () => {
    if (!task.trim()) return;
    const { host, port, model } = resolveTarget("planner");
    setDispatching(true);
    try {
      await ipc.agentStart({ host, port, model, task });
    } catch (e) {
      console.error(e);
    } finally {
      setDispatching(false);
      setTask("");
    }
  };

  const activeCount = state.agents.filter((a) => a.status === "Active").length;
  const parallel = config?.parallel ?? 1;
  const showWarn = parallel < 2 && activeCount > 1;

  return (
    <div className="flex flex-col h-full overflow-hidden p-3 gap-3">
      <div>
        <h2 className="text-sm font-semibold">Agent Monitor</h2>
        <p className="text-[10px] text-muted-foreground">
          Real-time activity and operational state of running agents.
        </p>
      </div>

      <div className="grid grid-cols-4 gap-2 flex-shrink-0">
        {[
          { label: "Registered", value: state.agents.length },
          { label: "Active", value: activeCount },
          { label: "Events", value: state.events.length },
          { label: "Parallel", value: parallel },
        ].map((s) => (
          <div key={s.label} className="glass-card text-center py-2">
            <p className="text-base font-bold tabular-nums">{s.value}</p>
            <p className="text-[9px] text-muted-foreground">{s.label}</p>
          </div>
        ))}
      </div>

      {showWarn && (
        <div className="flex items-start gap-2 text-[11px] glass-card p-2 rounded-md border-amber-500/30 text-amber-400">
          <AlertTriangle className="h-3.5 w-3.5 flex-shrink-0 mt-0.5" />
          <div>
            <strong>{activeCount} agents active, but parallel = {parallel}.</strong>{" "}
            Increase parallel slots in Settings → Server.
          </div>
        </div>
      )}

      <div className="glass-card p-3 flex-shrink-0">
        <p className="text-[11px] font-semibold mb-1.5">Dispatch Agent Task</p>
        <div className="flex gap-2">
          <Input
            value={task}
            onChange={(e) => setTask(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && dispatch()}
            placeholder="Enter a task for the agent…"
            className="flex-1 h-8 text-xs"
          />
          <Button size="sm" onClick={dispatch} disabled={dispatching || !task.trim()} className="h-8">
            {dispatching ? (
              <>
                <Activity className="h-3.5 w-3.5 animate-pulse" /> Starting…
              </>
            ) : (
              <>
                <Send className="h-3.5 w-3.5" /> Dispatch
              </>
            )}
          </Button>
        </div>
        <p className="text-[10px] text-muted-foreground mt-1">
          The agent will run on the routed/active model.
        </p>
      </div>

      <div className="glass-card p-3 flex flex-col flex-1 min-h-0">
        <p className="text-[11px] font-semibold mb-2 flex-shrink-0">Active Agent Processes</p>
        <AgentsList agents={state.agents} />
      </div>

      <div className="glass-card p-3 flex flex-col flex-1 min-h-0">
        <div className="flex items-center justify-between mb-2 flex-shrink-0">
          <p className="text-[11px] font-semibold">Execution Timeline</p>
          <Button size="sm" variant="ghost" onClick={clearLogs} className="h-6 text-[10px]">
            <Trash2 className="h-3 w-3" /> Clear Log
          </Button>
        </div>
        <EventsTimeline events={reversedEvents} />
      </div>
    </div>
  );
}

function BadgeDot({ status }: { status: string }) {
  const map: Record<string, string> = {
    Active: "bg-amber-500/20 text-amber-400 border-amber-500/40",
    Offline: "bg-muted text-muted-foreground border-muted",
    Idle: "bg-emerald-500/20 text-emerald-400 border-emerald-500/40",
  };
  const cls = map[status] ?? map.Idle;
  return (
    <span
      className={cn(
        "text-[8px] font-bold uppercase px-1 rounded border",
        cls
      )}
    >
      {status}
    </span>
  );
}

function AgentsList({ agents }: { agents: MonitorState["agents"] }) {
  const parentRef = useRef<HTMLDivElement>(null);
  const rowVirtualizer = useVirtualizer({
    count: agents.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 52,
    overscan: 6,
  });

  if (agents.length === 0) {
    return (
      <div className="flex-1 min-h-0 flex items-center justify-center">
        <p className="text-center text-[11px] text-muted-foreground italic">
          No registered agents. Dispatch a task above to start one.
        </p>
      </div>
    );
  }

  return (
    <div
      ref={parentRef}
      className="flex-1 min-h-0 overflow-auto pr-1"
    >
      <div
        style={{
          height: `${rowVirtualizer.getTotalSize()}px`,
          width: "100%",
          position: "relative",
        }}
      >
        {rowVirtualizer.getVirtualItems().map((virtualRow) => {
          const a = agents[virtualRow.index];
          const dotClass =
            a.status === "Active"
              ? "bg-amber-400 shadow-[0_0_4px_rgba(251,146,60,0.6)]"
              : a.status === "Offline"
              ? "bg-muted-foreground"
              : "bg-emerald-400";
          return (
            <div
              key={a.name}
              className="absolute inset-x-0 top-0 flex items-center gap-2 rounded-md border border-border/40 bg-muted/20 p-2"
              style={{ height: `${virtualRow.size}px`, transform: `translateY(${virtualRow.start}px)` }}
            >
              <Bot className="h-3.5 w-3.5 text-muted-foreground flex-shrink-0" />
              <div className={cn("h-2 w-2 rounded-full flex-shrink-0", dotClass)} />
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1.5">
                  <span className="font-semibold text-[11px]">{a.name}</span>
                  <BadgeDot status={a.status} />
                </div>
                <p className="text-[10px] text-muted-foreground truncate">
                  {a.current_action}
                </p>
              </div>
              <span className="text-[10px] text-muted-foreground flex-shrink-0">
                {a.last_active}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function EventsTimeline({ events }: { events: MonitorState["events"] }) {
  const parentRef = useRef<HTMLDivElement>(null);
  const rowVirtualizer = useVirtualizer({
    count: events.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 22,
    overscan: 30,
  });

  // Auto-scroll to top when new events arrive
  useEffect(() => {
    if (parentRef.current) {
      parentRef.current.scrollTop = 0;
    }
  }, [events.length]);

  return (
    <div className="flex-1 min-h-0 flex flex-col">
      <div className="grid grid-cols-[60px_100px_1fr] gap-1 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground border-b border-border/40 pb-1 mb-1 flex-shrink-0">
        <span>Time</span>
        <span>Agent</span>
        <span>Message</span>
      </div>
      {events.length === 0 ? (
        <div className="flex-1 flex items-center justify-center">
          <p className="text-center text-[11px] text-muted-foreground italic">
            Timeline is empty.
          </p>
        </div>
      ) : (
        <div ref={parentRef} className="flex-1 min-h-0 overflow-auto pr-1">
          <div
            style={{
              height: `${rowVirtualizer.getTotalSize()}px`,
              width: "100%",
              position: "relative",
            }}
          >
            {rowVirtualizer.getVirtualItems().map((virtualRow) => {
              const e = events[virtualRow.index];
              const ts = e.timestamp.length >= 19 ? e.timestamp.slice(11, 19) : e.timestamp;
              return (
                <div
                  key={virtualRow.key}
                  className="absolute inset-x-0 top-0 grid grid-cols-[60px_100px_1fr] gap-1 text-[10.5px] hover:bg-muted/20 px-1 py-0.5 rounded"
                  style={{ height: `${virtualRow.size}px`, transform: `translateY(${virtualRow.start}px)` }}
                >
                  <span className="font-mono text-muted-foreground">{ts}</span>
                  <span className="text-primary font-medium truncate">{e.agent_name}</span>
                  <span className="break-words">{e.message}</span>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
