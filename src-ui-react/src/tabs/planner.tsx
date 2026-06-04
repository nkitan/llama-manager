import { useState, useEffect } from "react";
import { Plus, Trash2, ChevronRight, ChevronLeft, GripVertical } from "lucide-react";
import { ipc, type KanbanTask } from "@/lib/ipc";
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

const PRIORITY_COLORS: Record<string, string> = {
  High: "text-destructive",
  Medium: "text-amber-400",
  Low: "text-muted-foreground",
};

export function PlannerTab() {
  const [tasks, setTasks] = useState<KanbanTask[]>([]);
  const [showAdd, setShowAdd] = useState(false);

  useEffect(() => {
    ipc.plannerLoad().then((s) => setTasks(s.tasks));
  }, []);

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
  };

  return (
    <div className="flex flex-col h-full overflow-hidden p-3 gap-3">
      {/* Header */}
      <div className="flex items-center justify-between flex-shrink-0">
        <span className="text-sm font-semibold">Task Planner</span>
        <Button size="sm" onClick={() => setShowAdd(true)} className="h-7 text-xs">
          <Plus className="h-3.5 w-3.5" /> New task
        </Button>
      </div>

      {/* Kanban board */}
      <div className="flex flex-1 min-h-0 gap-3 overflow-x-auto">
        {COLUMNS.map((col) => {
          const colTasks = tasks.filter((t) => t.status === col.id);
          return (
            <div key={col.id} className="flex flex-col glass-card w-60 flex-shrink-0 overflow-hidden">
              {/* Column header */}
              <div className="flex items-center gap-2 px-3 py-2 border-b border-border/30 flex-shrink-0">
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
                      onMove={(dir) => moveTask(task.id, dir)}
                      onDelete={() => deleteTask(task.id)}
                    />
                  ))}
                  {colTasks.length === 0 && (
                    <p className="text-center text-[11px] text-muted-foreground py-4">Empty</p>
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
  task, colIndex, totalCols, onMove, onDelete,
}: {
  task: KanbanTask;
  colIndex: number;
  totalCols: number;
  onMove: (dir: "left" | "right") => void;
  onDelete: () => void;
}) {
  return (
    <div className="glass-card p-2.5 space-y-2 group hover:border-border/60 transition-colors">
      <div className="flex items-start gap-1.5">
        <span className="text-[11px] font-medium flex-1 leading-tight">{task.title}</span>
        <button
          onClick={onDelete}
          className="opacity-0 group-hover:opacity-100 transition-opacity text-muted-foreground hover:text-destructive flex-shrink-0 no-drag"
        >
          <Trash2 className="h-3 w-3" />
        </button>
      </div>
      <p className="text-[10px] text-muted-foreground line-clamp-2">{task.summary}</p>
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-1.5">
          <span className={cn("text-[9px] font-semibold", PRIORITY_COLORS[task.priority] ?? "text-muted-foreground")}>
            {task.priority}
          </span>
          <span className="text-[9px] text-muted-foreground/60">·</span>
          <span className="text-[9px] text-muted-foreground">{task.assigned_agent}</span>
        </div>
        <div className="flex gap-0.5">
          {colIndex > 0 && (
            <button onClick={() => onMove("left")} className="text-muted-foreground hover:text-foreground no-drag">
              <ChevronLeft className="h-3.5 w-3.5" />
            </button>
          )}
          {colIndex < totalCols - 1 && (
            <button onClick={() => onMove("right")} className="text-muted-foreground hover:text-foreground no-drag">
              <ChevronRight className="h-3.5 w-3.5" />
            </button>
          )}
        </div>
      </div>
      <p className="text-[9px] text-muted-foreground/50">{task.created_at}</p>
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
