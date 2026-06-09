import { useState, useEffect } from "react";
import { Plus, Trash2, ChevronRight, ChevronLeft, GripVertical, Zap, X, Info, AlertTriangle } from "lucide-react";
import { ipc, onAgentEvent, type KanbanTask } from "@/lib/ipc";
import { useStore } from "@/store/app-store";
import { resolveTarget } from "@/lib/target";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";

type Status = KanbanTask["status"];
const COLUMNS: { id: Status; label: string; color: string }[] = [
  { id: "backlog",     label: "Backlog",     color: "text-muted-foreground" },
  { id: "todo",        label: "Todo",        color: "text-blue-400" },
  { id: "in_progress", label: "In Progress", color: "text-amber-400" },
  { id: "done",        label: "Done",        color: "text-emerald-400" },
];

const PRIORITY_STYLES: Record<string, { color: string; border: string }> = {
  High:   { color: "text-destructive",         border: "border-destructive/50" },
  Medium: { color: "text-amber-400",            border: "border-amber-400/50" },
  Low:    { color: "text-muted-foreground",    border: "border-muted-foreground/40" },
};

export function PlannerTab() {
  const [tasks, setTasks] = useState<KanbanTask[]>([]);
  const [showAdd, setShowAdd] = useState(false);
  const [showInfo, setShowInfo] = useState(false);
  const [runningCard, setRunningCard] = useState<string | null>(null);
  const [runningAgentId, setRunningAgentId] = useState<string | null>(null);
  const [agentOutput, setAgentOutput] = useState("");
  const config = useStore((s) => s.config);
  const addObsEvent = useStore((s) => s.addObsEvent);

  useEffect(() => {
    ipc.plannerLoad().then((s) => setTasks(s.tasks));
  }, []);

  // Listen to live agent events for the running card
  useEffect(() => {
    if (!runningAgentId) return;
    let unlisten: (() => void) | null = null;
    onAgentEvent((ev) => {
      if (ev.agent_id !== runningAgentId) return;
      if (ev.kind === "token") {
        setAgentOutput((o) => o + ev.delta);
      } else if (ev.kind === "step") {
        setAgentOutput((o) => o + `\n💭 ${ev.thought}\n`);
      } else if (ev.kind === "tool_call") {
        setAgentOutput((o) => o + `\n🔧 ${ev.call.tool}(${JSON.stringify(ev.call.args).slice(0, 80)})\n`);
      } else if (ev.kind === "tool_result") {
        const o = ev.result.output.length > 100 ? ev.result.output.slice(0, 100) + "..." : ev.result.output;
        setAgentOutput((s) => s + `  ${ev.result.ok ? "✅" : "⚠️"} ${o}\n`);
      } else if (ev.kind === "done") {
        setAgentOutput((s) => `Done ✓\n\nFinal Answer:\n${ev.final_text || s}`);
        setRunningCard(null);
        setRunningAgentId(null);
      } else if (ev.kind === "error") {
        setAgentOutput(`❌ Error:\n${ev.message}`);
        setRunningCard(null);
        setRunningAgentId(null);
      }
    }).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, [runningAgentId]);

  const save = async (updated: KanbanTask[]) => {
    await ipc.plannerSave(updated).catch(() => {});
    setTasks(updated);
  };

  const moveTask = async (id: string, direction: "left" | "right") => {
    const task = tasks.find((t) => t.id === id);
    if (!task) return;
    const cols = COLUMNS.map((c) => c.id);
    const idx = cols.indexOf(task.status);
    const newIdx = direction === "left" ? idx - 1 : idx + 1;
    if (newIdx < 0 || newIdx >= cols.length) return;
    await save(tasks.map((t) => t.id === id ? { ...t, status: cols[newIdx] } : t));
  };

  const deleteTask = async (id: string) => {
    await save(tasks.filter((t) => t.id !== id));
    if (runningCard === id) {
      setRunningCard(null);
      setRunningAgentId(null);
      setAgentOutput("");
    }
  };

  const dispatchAgent = async (task: KanbanTask) => {
    const prompt = `Complete this task: ${task.title} — ${task.summary}`;
    setRunningCard(task.id);
    setAgentOutput("⏳ Dispatching agent…\n");
    const { host, port, model } = resolveTarget("planner");
    try {
      const id = await ipc.agentStart({ host, port, model, task: prompt });
      setRunningAgentId(id);
      setAgentOutput((o) => o + `Agent started: ${id}\nListening for events…\n`);
      addObsEvent({ ts: Date.now(), kind: "agent:planner_dispatch", id: task.id, content: task.title });
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setAgentOutput(`Error: ${msg}`);
      setRunningCard(null);
    }
  };

  const showParallelWarn = (config?.parallel ?? 1) < 2 && tasks.filter((t) => t.status === "in_progress").length > 1;

  return (
    <div className="flex flex-col h-full overflow-hidden p-3 gap-3">
      {/* Header */}
      <div className="flex items-center justify-between flex-shrink-0">
        <div className="flex items-center gap-2">
          <span className="text-sm font-semibold">Task Planner</span>
          <span className="text-[11px] text-muted-foreground">AI Agent Kanban checklist integration</span>
          <button
            onClick={() => setShowInfo((v) => !v)}
            className="h-5 w-5 rounded-full border border-border/40 text-[10px] text-muted-foreground hover:text-foreground hover:border-primary transition-colors no-drag"
            title="About agent task integration"
          >
            i
          </button>
        </div>
        <Button size="sm" onClick={() => setShowAdd(true)} className="h-7 text-xs">
          <Plus className="h-3.5 w-3.5" /> New task
        </Button>
      </div>

      {showInfo && (
        <div className="glass-card border-primary/30 p-3 text-[11px] space-y-1.5 flex-shrink-0">
          <p className="font-semibold flex items-center gap-1.5">
            <Info className="h-3.5 w-3.5" />
            Referencing Planner Tasks in AI Agent
          </p>
          <p className="text-muted-foreground">
            The AI agent can load, modify, and coordinate your Kanban task list via the planner tools.
            You can also run agents directly on a card using the ⚡ button, or interact with prompts like:
          </p>
          <ul className="text-muted-foreground list-disc list-inside space-y-0.5">
            <li>{"\"Show me my current backlog of tasks in the planner.\""}</li>
            <li>{"\"Add a new high priority task 'Optimize search speed' to backlog.\""}</li>
            <li>{"\"Move the task 'Fix memory leak' to In Progress.\""}</li>
            <li>{"\"Tell me what tasks are currently in progress and summarize their status.\""}</li>
          </ul>
        </div>
      )}

      {showParallelWarn && (
        <div className="glass-card border-amber-500/40 p-2 text-[11px] text-amber-400 flex items-start gap-2 flex-shrink-0">
          <AlertTriangle className="h-3.5 w-3.5 flex-shrink-0 mt-0.5" />
          <div>
            <strong>Multiple tasks in progress, but parallelism is set to 1.</strong>{" "}
            Agents will queue and run serially. To run agents concurrently, increase{" "}
            <em>Parallel</em> slots in Settings → Advanced and consider using a smaller/quantized model.
          </div>
        </div>
      )}

      {/* Live agent output panel */}
      {runningCard && (
        <div className="glass-card border-primary/30 flex-shrink-0 overflow-hidden">
          <div className="flex items-center gap-2 px-3 py-2 border-b border-border/40 bg-primary/5">
            <div className="h-1.5 w-1.5 rounded-full bg-amber-400 animate-pulse" />
            <span className="text-[11px] font-semibold flex-1">Agent Running</span>
            <Button
              size="sm" variant="ghost"
              onClick={() => { setRunningCard(null); setRunningAgentId(null); setAgentOutput(""); }}
              className="h-5 text-[10px] no-drag"
            >
              <X className="h-3 w-3" /> Dismiss
            </Button>
          </div>
          <ScrollArea className="max-h-[180px]">
            <pre className="text-[10.5px] font-mono whitespace-pre-wrap break-words p-3">
              {agentOutput || "(waiting…)"}
            </pre>
          </ScrollArea>
        </div>
      )}

      {/* Kanban board */}
      <div className="flex flex-1 min-h-0 gap-3 overflow-x-auto">
        {COLUMNS.map((col) => {
          const colTasks = tasks.filter((t) => t.status === col.id);
          return (
            <div key={col.id} className="flex flex-col glass-card w-60 flex-shrink-0 overflow-hidden">
              {/* Column header */}
              <div className="flex items-center gap-2 px-3 py-2 border-b border-border/30 flex-shrink-0">
                <div className={cn("h-1.5 w-1.5 rounded-full",
                  col.id === "backlog" ? "bg-muted-foreground" :
                  col.id === "todo" ? "bg-blue-400" :
                  col.id === "in_progress" ? "bg-amber-400" :
                  "bg-emerald-400"
                )} />
                <span className={cn("text-xs font-semibold", col.color)}>{col.label}</span>
                <span className="ml-auto text-[10px] text-muted-foreground bg-muted rounded-full px-1.5">{colTasks.length}</span>
              </div>

              {/* Tasks */}
              <ScrollArea className="flex-1">
                <div className="p-2 space-y-2">
                  {colTasks.map((task) => (
                    <TaskCard
                      key={task.id}
                      task={task}
                      colIndex={COLUMNS.findIndex((c) => c.id === col.id)}
                      totalCols={COLUMNS.length}
                      isRunning={runningCard === task.id}
                      onMove={(dir) => moveTask(task.id, dir)}
                      onDelete={() => deleteTask(task.id)}
                      onExecute={() => dispatchAgent(task)}
                    />
                  ))}
                  {colTasks.length === 0 && (
                    <p className="text-center text-[11px] text-muted-foreground py-4">Drop tasks here</p>
                  )}
                </div>
              </ScrollArea>
            </div>
          );
        })}
      </div>

      {showAdd && (
        <AddTaskDialog
          onAdd={async (task) => {
            await save([...tasks, task]);
            setShowAdd(false);
          }}
          onClose={() => setShowAdd(false)}
        />
      )}
    </div>
  );
}

function TaskCard({
  task, colIndex, totalCols, isRunning, onMove, onDelete, onExecute,
}: {
  task: KanbanTask;
  colIndex: number;
  totalCols: number;
  isRunning: boolean;
  onMove: (dir: "left" | "right") => void;
  onDelete: () => void;
  onExecute: () => void;
}) {
  const pri = PRIORITY_STYLES[task.priority] ?? PRIORITY_STYLES.Medium;
  return (
    <div className={cn(
      "glass-card p-2.5 space-y-2 group hover:border-border/60 transition-colors",
      isRunning && "ring-1 ring-primary/50"
    )}>
      <div className="flex items-start gap-1.5">
        <span className="text-[11px] font-medium flex-1 leading-tight">{task.title}</span>
        <button
          onClick={onDelete}
          className="opacity-0 group-hover:opacity-100 transition-opacity text-muted-foreground hover:text-destructive flex-shrink-0 no-drag"
        >
          <Trash2 className="h-3 w-3" />
        </button>
      </div>
      {task.summary && (
        <p className="text-[10px] text-muted-foreground line-clamp-2">{task.summary}</p>
      )}
      <Button
        size="sm"
        onClick={onExecute}
        disabled={isRunning}
        className="w-full h-6 text-[10.5px] gap-1"
      >
        {isRunning ? (
          <><div className="h-2.5 w-2.5 rounded-full border border-current border-t-transparent animate-spin" /> Running…</>
        ) : (
          <><Zap className="h-3 w-3" /> Execute Now</>
        )}
      </Button>
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-1.5 min-w-0">
          {task.assigned_agent ? (
            <span className="text-[9px] text-muted-foreground truncate">🤖 {task.assigned_agent}</span>
          ) : (
            <span className="text-[9px] text-muted-foreground/60">—</span>
          )}
        </div>
        <div className="flex items-center gap-1.5">
          <span className={cn("text-[9px] font-semibold border rounded px-1", pri.color, pri.border)}>
            {task.priority}
          </span>
          <span className="text-[9px] text-muted-foreground/60">{task.created_at}</span>
        </div>
      </div>
      <div className="flex items-center justify-between">
        <div className="flex gap-0.5">
          {colIndex > 0 && (
            <button onClick={() => onMove("left")} className="text-muted-foreground hover:text-foreground no-drag" title="Previous column">
              <ChevronLeft className="h-3.5 w-3.5" />
            </button>
          )}
          {colIndex < totalCols - 1 && (
            <button onClick={() => onMove("right")} className="text-muted-foreground hover:text-foreground no-drag" title="Next column">
              <ChevronRight className="h-3.5 w-3.5" />
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

function AddTaskDialog({
  onAdd, onClose,
}: {
  onAdd: (task: KanbanTask) => Promise<void>;
  onClose: () => void;
}) {
  const [title, setTitle] = useState("");
  const [summary, setSummary] = useState("");
  const [agent, setAgent] = useState("");
  const [priority, setPriority] = useState<"Low" | "Medium" | "High">("Medium");
  const [status, setStatus] = useState<Status>("todo");
  const [saving, setSaving] = useState(false);

  const submit = async () => {
    if (!title.trim() || !summary.trim()) return;
    setSaving(true);
    const task: KanbanTask = {
      id: `task-${Date.now()}`,
      title: title.trim(),
      summary: summary.trim(),
      assigned_agent: agent.trim() || "Agent",
      status,
      priority,
      created_at: new Date().toISOString().slice(0, 10),
    };
    await onAdd(task);
    setSaving(false);
  };

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogHeader><DialogTitle>New Task</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div className="space-y-1">
            <Label className="text-xs">Title</Label>
            <Input value={title} onChange={(e) => setTitle(e.target.value)} placeholder="Implement OAuth2 login" />
          </div>
          <div className="space-y-1">
            <Label className="text-xs">Summary</Label>
            <textarea
              value={summary}
              onChange={(e) => setSummary(e.target.value)}
              rows={2}
              placeholder="Detailed description or acceptance criteria…"
              className="flex w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring resize-none no-drag"
            />
          </div>
          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1">
              <Label className="text-xs">Priority</Label>
              <Select value={priority} onValueChange={(v) => setPriority(v as typeof priority)}>
                <SelectTrigger className="h-8 text-xs"><SelectValue /></SelectTrigger>
                <SelectContent>
                  {["Low", "Medium", "High"].map((p) => <SelectItem key={p} value={p} className="text-xs">{p}</SelectItem>)}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-1">
              <Label className="text-xs">Initial column</Label>
              <Select value={status} onValueChange={(v) => setStatus(v as Status)}>
                <SelectTrigger className="h-8 text-xs"><SelectValue /></SelectTrigger>
                <SelectContent>
                  {COLUMNS.map((c) => <SelectItem key={c.id} value={c.id} className="text-xs">{c.label}</SelectItem>)}
                </SelectContent>
              </Select>
            </div>
          </div>
          <div className="space-y-1">
            <Label className="text-xs">Assigned agent</Label>
            <Input value={agent} onChange={(e) => setAgent(e.target.value)} placeholder="Developer, Researcher…" />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>Cancel</Button>
          <Button onClick={submit} disabled={saving || !title.trim() || !summary.trim()}>
            {saving ? "Creating…" : "Create task"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
