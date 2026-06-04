import { useEffect, useState, useMemo } from "react";
import { Save, ChevronDown, ChevronUp, FileJson, Server } from "lucide-react";
import { ipc } from "@/lib/ipc";
import { useStore } from "@/store/app-store";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";

type ServerInfo = {
  name: string;
  command: string;
  args: string;
  envKeys: string[];
  tools: string[];
};

const BUILTIN_TOOLS: { icon: string; name: string; desc: string }[] = [
  { icon: "📋", name: "get_todos", desc: "Retrieve current user todo checklist." },
  { icon: "📋", name: "add_todo", desc: "Add a new todo item to the checklist." },
  { icon: "📋", name: "complete_todo", desc: "Mark a todo item as completed." },
  { icon: "📝", name: "read_notes", desc: "Read global and project notes content." },
  { icon: "📝", name: "write_note", desc: "Create or update a note with content." },
  { icon: "📅", name: "get_calendar_events", desc: "Retrieve scheduled calendar events." },
  { icon: "📅", name: "add_calendar_event", desc: "Add a scheduled event to the calendar." },
  { icon: "📅", name: "delete_calendar_event", desc: "Delete a calendar event by ID." },
  { icon: "🗂", name: "get_planner_tasks", desc: "Read tasks on the Kanban board." },
  { icon: "🗂", name: "add_planner_task", desc: "Create a new Kanban task card." },
  { icon: "🗂", name: "update_planner_task", desc: "Update a Kanban task card's details/status." },
  { icon: "🗂", name: "delete_planner_task", desc: "Delete a task card from the planner." },
  { icon: "🔍", name: "web_search", desc: "Query SearXNG search index." },
  { icon: "📄", name: "web_scrape", desc: "Extract text content from any URL." },
  { icon: "💻", name: "run_command", desc: "Execute bash commands on host (Sensitive)." },
  { icon: "🔌", name: "call_mcp_tool", desc: "Call external Model Context Protocol tool." },
  { icon: "🧠", name: "spawn_subagent", desc: "Delegate a subtask to an autonomous sub-agent." },
];

export function McpTab() {
  const toolsCollapsed = useStore((s) => s.toolsCollapsed);
  const toggleToolsCollapsed = useStore((s) => s.toggleToolsCollapsed);
  const [raw, setRaw] = useState("");
  const [status, setStatus] = useState("");
  const [showRaw, setShowRaw] = useState(false);

  const load = async () => {
    try {
      const val = await ipc.mcpGetRegistry();
      setRaw(JSON.stringify(val, null, 2));
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setStatus(`Load failed: ${msg}`);
    }
  };
  useEffect(() => { load(); }, []);

  const save = async () => {
    setStatus("Saving…");
    let json: unknown;
    try {
      json = JSON.parse(raw);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setStatus(`Invalid JSON: ${msg}`);
      return;
    }
    try {
      await ipc.mcpSaveRegistry(json as Record<string, unknown>);
      setStatus("Saved ✓");
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setStatus(`Error: ${msg}`);
    }
  };

  const servers = useMemo<ServerInfo[]>(() => {
    try {
      const v = JSON.parse(raw) as Record<string, unknown>;
      const mcpServers = v.mcpServers as Record<string, Record<string, unknown>> | undefined;
      if (!mcpServers) return [];
      return Object.entries(mcpServers).map(([name, sv]) => ({
        name,
        command: (sv.command as string) ?? "",
        args: Array.isArray(sv.args) ? sv.args.filter((a): a is string => typeof a === "string").join(" ") : "",
        envKeys: sv.env && typeof sv.env === "object" && !Array.isArray(sv.env)
          ? Object.keys(sv.env)
          : [],
        tools: Array.isArray(sv.tools) ? sv.tools.filter((t): t is string => typeof t === "string") : [],
      }));
    } catch {
      return [];
    }
  }, [raw]);

  return (
    <div className="flex flex-col h-full overflow-hidden p-3 gap-3">
      <div>
        <h2 className="text-sm font-semibold">MCP Tools</h2>
        <p className="text-[10px] text-muted-foreground">
          Model Context Protocol server registry and configuration.
        </p>
      </div>

      {status && (
        <div className="glass-card p-2 text-[11px] flex-shrink-0">
          {status}
        </div>
      )}

      <div className="flex items-center gap-2 flex-shrink-0">
        <Button size="sm" onClick={save} className="h-7 text-[11px]">
          <Save className="h-3 w-3" /> Save Registry
        </Button>
        <Button size="sm" variant="outline" onClick={() => setShowRaw((v) => !v)} className="h-7 text-[11px]">
          <FileJson className="h-3 w-3" /> {showRaw ? "Card View" : "Raw JSON"}
        </Button>
      </div>

      {showRaw ? (
        <div className="glass-card p-3 flex-1 min-h-0 flex flex-col gap-2">
          <p className="text-[11px] font-semibold">Raw MCP Registry JSON</p>
          <Textarea
            value={raw}
            onChange={(e) => setRaw(e.target.value)}
            className="flex-1 font-mono text-[12.5px] resize-none"
          />
        </div>
      ) : (
        <ScrollArea className="flex-1 min-h-0">
          <div className="space-y-3 pr-1">
            <div className="glass-card overflow-hidden">
              <button
                onClick={toggleToolsCollapsed}
                className="w-full flex items-center gap-2 px-3 py-2.5 hover:bg-muted/30 transition-colors"
              >
                <span className="text-[12.5px] font-semibold flex-1 text-left">🔧 Built-in Agent Tools</span>
                <span className="text-[10px] text-muted-foreground">
                  {toolsCollapsed ? "(click to expand)" : "(click to collapse)"}
                </span>
                {toolsCollapsed ? (
                  <ChevronDown className="h-3.5 w-3.5" />
                ) : (
                  <ChevronUp className="h-3.5 w-3.5" />
                )}
              </button>
              {!toolsCollapsed && (
                <div className="border-t border-border/40 px-3 py-3 space-y-2">
                  <p className="text-[10.5px] text-muted-foreground">
                    These core tools are built directly into the agent runner,
                    providing autonomous capabilities to inspect/update your
                    planner, todos, notes, calendar, and model memory.
                  </p>
                  <div className="grid grid-cols-[repeat(auto-fill,minmax(240px,1fr))] gap-2">
                    {BUILTIN_TOOLS.map((t) => (
                      <div key={t.name} className="rounded-md border border-border/40 bg-muted/30 p-2 space-y-0.5">
                        <div className="font-semibold text-[11px] flex items-center gap-1.5">
                          <span>{t.icon}</span> {t.name}
                        </div>
                        <p className="text-[10px] text-muted-foreground">{t.desc}</p>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>

            <div>
              <p className="text-[12.5px] font-semibold mb-1.5">Active MCP Servers</p>
              {servers.length === 0 ? (
                <p className="text-center text-[11px] text-muted-foreground italic py-8">
                  No MCP servers configured. Switch to Raw JSON editor to add servers.
                </p>
              ) : (
                <div className="grid grid-cols-[repeat(auto-fill,minmax(280px,1fr))] gap-2.5">
                  {servers.map((s) => (
                    <div key={s.name} className="rounded-md border border-border/40 bg-muted/20 p-2.5 space-y-1.5">
                      <div className="flex items-center gap-1.5">
                        <Server className="h-3.5 w-3.5 text-muted-foreground" />
                        <span className="font-semibold text-[12px] flex-1">{s.name}</span>
                        <span className="text-[9px] font-bold uppercase px-1.5 py-0.5 rounded bg-emerald-500/20 text-emerald-400 border border-emerald-500/40">
                          Running
                        </span>
                      </div>
                      <p
                        className="text-[10.5px] font-mono text-muted-foreground truncate"
                        title={`${s.command} ${s.args}`}
                      >
                        {s.command} {s.args}
                      </p>
                      {s.envKeys.length > 0 && (
                        <div className="text-[10px] text-muted-foreground flex flex-wrap gap-1">
                          <span>Env:</span>
                          {s.envKeys.map((k) => (
                            <span key={k} className="text-[9px] font-mono px-1 py-0.5 rounded bg-muted/60 border border-border/40">
                              {k}
                            </span>
                          ))}
                        </div>
                      )}
                      {s.tools.length > 0 && (
                        <div>
                          <p className="text-[9px] font-bold uppercase tracking-wider text-muted-foreground mb-1">Tools</p>
                          <div className="flex flex-wrap gap-1">
                            {s.tools.map((t) => (
                              <span
                                key={t}
                                className="text-[9px] font-mono px-1.5 py-0.5 rounded bg-primary/20 text-primary border border-primary/30"
                              >
                                🔧 {t}
                              </span>
                            ))}
                          </div>
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </ScrollArea>
      )}
    </div>
  );
}
