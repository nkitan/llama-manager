import { useState, useEffect } from "react";
import { Plus, Check, Trash2, Circle } from "lucide-react";
import { ipc, type TodoItem } from "@/lib/ipc";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

export function TodosTab() {
  const [todos, setTodos] = useState<TodoItem[]>([]);
  const [draft, setDraft] = useState("");
  const [filter, setFilter] = useState<"all" | "active" | "done">("all");

  useEffect(() => {
    ipc.todosGet().then(setTodos);
  }, []);

  const save = async (items: TodoItem[]) => {
    await ipc.todosSet(items).catch(() => {});
    setTodos(items);
  };

  const add = async () => {
    if (!draft.trim()) return;
    const item: TodoItem = {
      id: `t${Date.now()}`,
      text: draft.trim(),
      done: false,
      priority: "",
      due_date: null,
      tags: [],
    };
    await save([...todos, item]);
    setDraft("");
  };

  const toggle = (id: string) =>
    save(todos.map((t) => (t.id === id ? { ...t, done: !t.done } : t)));

  const remove = (id: string) => save(todos.filter((t) => t.id !== id));

  const clearDone = () => save(todos.filter((t) => !t.done));

  const filtered = todos.filter((t) => {
    if (filter === "active") return !t.done;
    if (filter === "done") return t.done;
    return true;
  });

  const doneCount = todos.filter((t) => t.done).length;

  return (
    <div className="flex flex-col h-full overflow-hidden p-3 gap-3">
      {/* Header */}
      <div className="flex items-center justify-between flex-shrink-0">
        <div className="flex items-center gap-2">
          <span className="text-sm font-semibold">Todos</span>
          <Badge variant="secondary" className="text-[10px]">
            {todos.length - doneCount} remaining
          </Badge>
        </div>
        {doneCount > 0 && (
          <Button variant="ghost" size="sm" onClick={clearDone} className="text-xs h-7 text-muted-foreground">
            Clear {doneCount} done
          </Button>
        )}
      </div>

      {/* Filter */}
      <div className="flex gap-1 flex-shrink-0">
        {(["all", "active", "done"] as const).map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={cn(
              "rounded-full px-2.5 py-0.5 text-[11px] font-medium capitalize transition-colors no-drag",
              filter === f ? "bg-primary text-primary-foreground" : "bg-muted text-muted-foreground hover:text-foreground"
            )}
          >
            {f}
          </button>
        ))}
      </div>

      {/* Add input */}
      <div className="flex gap-2 flex-shrink-0">
        <Input
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && add()}
          placeholder="Add a todo… (Enter)"
          className="flex-1 text-sm h-9"
        />
        <Button size="icon" onClick={add} disabled={!draft.trim()}>
          <Plus className="h-4 w-4" />
        </Button>
      </div>

      {/* List */}
      <ScrollArea className="flex-1">
        <div className="space-y-1">
          {filtered.length === 0 && (
            <p className="text-center text-sm text-muted-foreground py-8">
              {filter === "done" ? "No completed todos" : "All done! 🎉"}
            </p>
          )}
          {filtered.map((todo) => (
            <div
              key={todo.id}
              className={cn(
                "group flex items-center gap-2.5 glass-card rounded-md px-3 py-2.5 transition-all",
                todo.done && "opacity-60"
              )}
            >
              <button
                onClick={() => toggle(todo.id)}
                className={cn(
                  "flex-shrink-0 h-5 w-5 rounded-full border-2 flex items-center justify-center transition-all no-drag",
                  todo.done ? "bg-primary border-primary text-primary-foreground" : "border-border hover:border-primary"
                )}
              >
                {todo.done ? <Check className="h-3 w-3" /> : null}
              </button>
              <span className={cn("flex-1 text-sm", todo.done && "line-through text-muted-foreground")}>
                {todo.text}
              </span>
              <button
                onClick={() => remove(todo.id)}
                className="opacity-0 group-hover:opacity-100 transition-opacity no-drag text-muted-foreground hover:text-destructive"
              >
                <Trash2 className="h-3.5 w-3.5" />
              </button>
            </div>
          ))}
        </div>
      </ScrollArea>

      {/* Progress bar */}
      {todos.length > 0 && (
        <div className="flex-shrink-0 space-y-1">
          <div className="flex justify-between text-[10px] text-muted-foreground">
            <span>{doneCount} / {todos.length} completed</span>
            <span>{Math.round((doneCount / todos.length) * 100)}%</span>
          </div>
          <div className="h-1.5 rounded-full bg-primary/20 overflow-hidden">
            <div
              className="h-full bg-primary rounded-full transition-all"
              style={{ width: `${(doneCount / todos.length) * 100}%` }}
            />
          </div>
        </div>
      )}
    </div>
  );
}
