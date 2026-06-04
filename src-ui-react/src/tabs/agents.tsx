import { useEffect, useMemo, useState } from "react";
import {
  Plus, Trash2, Save, X, RefreshCw, Bot, Network, FileCode2,
  Sparkles, LinkIcon, Tag, Globe, FolderOpen, Search,
} from "lucide-react";
import { ipc, type Memory, type SkillOrAgentFile } from "@/lib/ipc";
import { useStore } from "@/store/app-store";
import { resolveTarget } from "@/lib/target";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Switch } from "@/components/ui/switch";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";

type ViewMode = "memory" | "skills";

export function AgentsTab() {
  const [view, setView] = useState<ViewMode>("memory");
  return (
    <div className="flex flex-col h-full overflow-hidden p-3 gap-3">
      <div className="flex items-center justify-between gap-2 flex-wrap">
        <div>
          <h2 className="text-sm font-semibold">Memory & Agent Knowledge Base</h2>
          <p className="text-[10px] text-muted-foreground">
            Persistent memory notes and discovered agent / skill rule files.
          </p>
        </div>
        <div className="flex gap-1 glass-card p-1 rounded-md">
          <ViewPill current={view === "memory"} onClick={() => setView("memory")} icon={<Network className="h-3 w-3" />}>Memory</ViewPill>
          <ViewPill current={view === "skills"} onClick={() => setView("skills")} icon={<FileCode2 className="h-3 w-3" />}>AGENTS.md / Skills</ViewPill>
        </div>
      </div>
      <div className="flex-1 min-h-0">
        {view === "memory" ? <MemoryWorkspace /> : <SkillsViewer />}
      </div>
    </div>
  );
}

function ViewPill({
  current, onClick, icon, children,
}: { current: boolean; onClick: () => void; icon: React.ReactNode; children: React.ReactNode }) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "flex items-center gap-1.5 px-2.5 h-6 rounded text-[11px] font-semibold transition-colors",
        current ? "bg-primary text-primary-foreground" : "hover:bg-muted/40 text-muted-foreground"
      )}
    >
      {icon}
      {children}
    </button>
  );
}

// ── Memory workspace ────────────────────────────────────────────────────────
function MemoryWorkspace() {
  const [memories, setMemories] = useState<Memory[]>([]);
  const [search, setSearch] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [isEditing, setIsEditing] = useState(false);
  const config = useStore((s) => s.config);

  const reload = async () => {
    try {
      const list = await ipc.obsidianMemoriesGet();
      setMemories(list);
    } catch (e) {
      console.error(e);
    }
  };
  useEffect(() => { reload(); }, []);

  const filtered = useMemo(() => {
    const q = search.toLowerCase();
    if (!q) return memories;
    return memories.filter(
      (m) =>
        m.title.toLowerCase().includes(q) ||
        m.content.toLowerCase().includes(q) ||
        m.tags.some((t) => t.toLowerCase().includes(q))
    );
  }, [memories, search]);

  const selected = selectedId ? memories.find((m) => m.id === selectedId) ?? null : null;

  const startNew = () => {
    setSelectedId(null);
    setIsEditing(true);
  };
  const editSelected = () => {
    if (selected) setIsEditing(true);
  };
  const close = () => {
    setSelectedId(null);
    setIsEditing(false);
  };

  const handleDelete = async (m: Memory) => {
    if (!confirm(`Delete "${m.title}"?`)) return;
    try {
      await ipc.obsidianMemoryDelete(m.id, m.scope);
      await reload();
      if (selectedId === m.id) close();
    } catch (e) {
      console.error(e);
    }
  };

  const handleClearAll = async () => {
    if (!confirm(`Delete all ${memories.length} memories? This cannot be undone.`)) return;
    for (const m of memories) {
      try { await ipc.obsidianMemoryDelete(m.id, m.scope); } catch {}
    }
    await reload();
    close();
  };

  const showParallelWarn = (config?.parallel ?? 1) < 2;
  const globals = filtered.filter((m) => m.scope === "global");
  const projects = filtered.filter((m) => m.scope === "project");

  return (
    <div className="flex gap-3 h-full min-h-0">
      <div className="w-[260px] flex-shrink-0 glass-card p-2 flex flex-col min-h-0">
        <div className="flex items-center gap-1.5 mb-1.5 px-1">
          <span className="text-[11px] font-semibold flex-1">Knowledge Nodes</span>
          <Button size="sm" variant="ghost" onClick={reload} className="h-6 w-6 p-0">
            <RefreshCw className="h-3 w-3" />
          </Button>
        </div>
        <div className="relative mb-1.5">
          <Search className="absolute left-2 top-1/2 -translate-y-1/2 h-3 w-3 text-muted-foreground" />
          <Input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Filter nodes…"
            className="h-7 text-[11px] pl-7"
          />
        </div>
        <ScrollArea className="flex-1 min-h-0">
          <ScopeGroup icon={<Globe className="h-3 w-3" />} label="Global" memories={globals} selectedId={selectedId} editing={isEditing} onSelect={setSelectedId} />
          <ScopeGroup icon={<FolderOpen className="h-3 w-3" />} label="Project" memories={projects} selectedId={selectedId} editing={isEditing} onSelect={setSelectedId} />
        </ScrollArea>
        <div className="flex flex-col gap-1 pt-1.5 border-t border-border/40 mt-1.5">
          <Button size="sm" variant="default" onClick={startNew} className="h-7 text-[11px] w-full">
            <Plus className="h-3 w-3" /> New Node
          </Button>
          <Button
            size="sm" variant="destructive"
            disabled={memories.length === 0}
            onClick={handleClearAll}
            className="h-7 text-[11px] w-full"
          >
            <Trash2 className="h-3 w-3" /> Clear All
          </Button>
        </div>
      </div>

      <div className="flex-1 flex flex-col min-h-0 gap-2">
        {showParallelWarn && (
          <div className="glass-card border-amber-500/40 text-amber-400 p-2 text-[11px] flex items-start gap-2">
            <Sparkles className="h-3.5 w-3.5 flex-shrink-0 mt-0.5" />
            <div>
              <strong>Single-threaded runtime check:</strong> Model server
              parallelism is set to 1. Concurrent agents or sub-agents will run
              serially. Increase "Parallel" slots in Settings → Server to
              enable multi-agent flows.
            </div>
          </div>
        )}

        {(selected || isEditing) ? (
          <MemoryEditor
            memory={selected}
            onSaved={async () => { await reload(); setIsEditing(false); }}
            onClose={close}
            onDelete={selected ? () => handleDelete(selected) : undefined}
          />
        ) : (
          <EmptyState onCreate={startNew} />
        )}

        {selected && <MemoryGraph memories={memories} selectedId={selected.id} onSelect={setSelectedId} />}
      </div>
    </div>
  );
}

function ScopeGroup({
  icon, label, memories, selectedId, editing, onSelect,
}: {
  icon: React.ReactNode; label: string; memories: Memory[];
  selectedId: string | null; editing: boolean;
  onSelect: (id: string) => void;
}) {
  return (
    <div className="mb-2">
      <div className="flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-wider text-muted-foreground px-1 py-1">
        {icon}{label}
      </div>
      {memories.length === 0 ? (
        <p className="text-[10px] text-muted-foreground italic px-2 py-1">Empty</p>
      ) : (
        memories.map((m) => (
          <button
            key={m.id}
            onClick={() => onSelect(m.id)}
            className={cn(
              "w-full text-left flex items-center gap-1.5 px-2 py-1 rounded text-[11px] hover:bg-muted/40",
              selectedId === m.id && !editing && "bg-muted/60"
            )}
          >
            <span className="text-[10px]">{m.scope === "global" ? "🌐" : "📝"}</span>
            <span className="truncate flex-1">{m.title || "Untitled"}</span>
          </button>
        ))
      )}
    </div>
  );
}

function MemoryEditor({
  memory, onSaved, onClose, onDelete,
}: {
  memory: Memory | null;
  onSaved: () => void;
  onClose: () => void;
  onDelete?: () => void;
}) {
  const initialId = memory?.id ?? `mem-${Date.now()}`;
  const [id] = useState(initialId);
  const [title, setTitle] = useState(memory?.title ?? "");
  const [scope, setScope] = useState(memory?.scope ?? "project");
  const [tagsInput, setTagsInput] = useState(memory?.tags.join(", ") ?? "");
  const [content, setContent] = useState(memory?.content ?? "");
  const [links, setLinks] = useState(memory?.links ?? []);
  const [agentPrompt, setAgentPrompt] = useState("");
  const [agentOutput, setAgentOutput] = useState("");
  const [agentRunning, setAgentRunning] = useState(false);

  const tagList = useMemo(
    () => tagsInput.split(",").map((s) => s.trim()).filter(Boolean),
    [tagsInput]
  );

  const save = async () => {
    const mem: Memory = {
      id,
      title: title.trim() || "Untitled",
      created_at: memory?.created_at ?? new Date().toISOString(),
      tags: tagList,
      links,
      scope,
      content,
    };
    try {
      await ipc.obsidianMemorySave(mem);
      onSaved();
    } catch (e) {
      console.error(e);
    }
  };

  const dispatchAgent = async () => {
    const task = agentPrompt.trim()
      ? `For memory note titled "${title}", perform this command: ${agentPrompt}\n\nExisting content:\n${content}`
      : `Analyze, expand, and summarize this memory note titled "${title}":\n\n${content}`;
    setAgentRunning(true);
    setAgentOutput("⏳ Dispatching agent copilot…");
    const { host, port, model } = resolveTarget("memory");
    try {
      const agentId = await ipc.agentStart({ host, port, model, task });
      setAgentOutput(`Agent started: ${agentId}. Watch the Monitor tab. Once finished, use the controls below to insert/append.`);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setAgentOutput(`Error starting agent: ${msg}`);
    } finally {
      setAgentRunning(false);
    }
  };

  const appendOutput = () => {
    if (!agentOutput || agentOutput.startsWith("⏳") || agentOutput.startsWith("E")) return;
    setContent((c) => `${c}\n\n### Agent Summary\n${agentOutput}`);
    setAgentOutput("");
  };

  return (
    <div className="glass-card flex-1 flex flex-col min-h-0 overflow-hidden">
      <div className="flex items-center gap-1.5 p-2 border-b border-border/40 flex-shrink-0">
        <Button size="sm" onClick={save} className="h-7 text-[11px]">
          <Save className="h-3 w-3" /> Save
        </Button>
        {onDelete && (
          <Button size="sm" variant="destructive" onClick={onDelete} className="h-7 text-[11px]">
            <Trash2 className="h-3 w-3" /> Delete
          </Button>
        )}
        <div className="flex-1" />
        <Button size="sm" variant="ghost" onClick={onClose} className="h-7 text-[11px]">
          <X className="h-3 w-3" /> Close
        </Button>
      </div>

      <div className="flex items-center gap-3 px-3 py-2 border-b border-border/40 flex-shrink-0 text-[11px]">
        <label className="flex items-center gap-1.5">
          <span className="text-muted-foreground font-semibold">Scope</span>
          <select
            value={scope}
            onChange={(e) => setScope(e.target.value)}
            className="h-6 text-[11px] bg-transparent border border-input rounded px-1 no-drag"
          >
            <option value="project">📂 Project</option>
            <option value="global">🌐 Global</option>
          </select>
        </label>
        <label className="flex items-center gap-1.5 flex-1 min-w-0">
          <Tag className="h-3 w-3 text-muted-foreground" />
          <Input
            value={tagsInput}
            onChange={(e) => setTagsInput(e.target.value)}
            placeholder="rust, research, …"
            className="h-6 text-[11px]"
          />
        </label>
        <div className="flex flex-wrap gap-1">
          {tagList.map((t) => (
            <span key={t} className="text-[9px] font-semibold px-1.5 py-0.5 rounded bg-primary/20 text-primary border border-primary/30">
              {t}
            </span>
          ))}
        </div>
      </div>

      <div className="px-3 py-2 border-b border-border/40 flex-shrink-0 space-y-1.5">
        <div className="flex items-center gap-1.5">
          <Bot className="h-3 w-3 text-muted-foreground" />
          <span className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground flex-1">
            Copilot Agent Instruction
          </span>
          {agentRunning ? (
            <span className="text-[10px] text-muted-foreground animate-pulse">running…</span>
          ) : (
            <Button size="sm" onClick={dispatchAgent} className="h-6 text-[10px]">
              Ask Agent
            </Button>
          )}
        </div>
        <Input
          value={agentPrompt}
          onChange={(e) => setAgentPrompt(e.target.value)}
          placeholder="Instruct the agent (e.g. 'summarize', 'find actions', leave blank for default analysis)"
          className="h-7 text-[11px]"
        />
        {agentOutput && (
          <div className="space-y-1.5">
            <pre className="text-[10.5px] bg-muted/40 rounded p-2 max-h-[100px] overflow-y-auto whitespace-pre-wrap font-mono">
              {agentOutput}
            </pre>
            {!agentOutput.startsWith("⏳") && !agentOutput.startsWith("E") && (
              <div className="flex gap-1.5">
                <Button size="sm" variant="outline" onClick={appendOutput} className="h-6 text-[10px]">
                  <Plus className="h-2.5 w-2.5" /> Append to Note
                </Button>
                <Button size="sm" variant="ghost" onClick={() => setAgentOutput("")} className="h-6 text-[10px]">
                  <X className="h-2.5 w-2.5" /> Dismiss
                </Button>
              </div>
            )}
          </div>
        )}
      </div>

      <div className="px-3 py-2 flex-shrink-0">
        <Input
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder="Untitled Memory"
          className="h-9 text-[14px] font-semibold border-0 px-0 focus-visible:ring-0 bg-transparent"
        />
      </div>

      <div className="flex-1 min-h-0 px-3 pb-2">
        <Textarea
          value={content}
          onChange={(e) => setContent(e.target.value)}
          placeholder="Write Markdown here…"
          className="h-full resize-none text-[12.5px] font-mono leading-relaxed"
        />
      </div>

      <div className="border-t border-border/40 p-2 max-h-[120px] overflow-y-auto flex-shrink-0">
        <div className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground mb-1.5">
          Links
        </div>
        {links.length === 0 ? (
          <p className="text-[10px] text-muted-foreground italic">No linked nodes. Toggle from the graph panel.</p>
        ) : (
          <div className="flex flex-wrap gap-1.5">
            {links.map((lid) => (
              <span key={lid} className="text-[10px] px-1.5 py-0.5 rounded bg-muted/60 border border-border/60 font-mono">
                {lid}
              </span>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function EmptyState({ onCreate }: { onCreate: () => void }) {
  return (
    <div className="glass-card flex-1 flex flex-col items-center justify-center gap-3 text-muted-foreground">
      <div className="text-5xl opacity-30">📓</div>
      <div className="text-[13px] font-semibold">Select a note or create a new one</div>
      <div className="text-[11px]">Your agent's accumulated knowledge lives here.</div>
      <Button size="sm" onClick={onCreate} className="mt-2">
        <Plus className="h-3.5 w-3.5" /> New Memory Node
      </Button>
    </div>
  );
}

function MemoryGraph({
  memories, selectedId, onSelect,
}: { memories: Memory[]; selectedId: string; onSelect: (id: string) => void }) {
  const W = 540, H = 320;
  const nodes = useMemo(() => {
    if (memories.length === 0) return [];
    const total = memories.length;
    return memories.map((m, i) => {
      const angle = (i / total) * 2 * Math.PI;
      const r = Math.min(W, H) * 0.32;
      return {
        id: m.id,
        title: m.title || "Untitled",
        scope: m.scope,
        x: W / 2 + r * Math.cos(angle),
        y: H / 2 + r * Math.sin(angle),
      };
    });
  }, [memories]);

  const edges = useMemo(() => {
    const ids = new Set(memories.map((m) => m.id));
    const out: { source: string; target: string }[] = [];
    for (const m of memories) {
      for (const l of m.links) {
        if (ids.has(l) && ids.has(m.id)) out.push({ source: m.id, target: l });
      }
    }
    return out;
  }, [memories]);

  if (memories.length === 0) return null;

  return (
    <div className="glass-card p-2 h-[280px] flex-shrink-0">
      <div className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground mb-1 px-1">
        <Network className="inline h-3 w-3 mr-1" /> Knowledge Graph
      </div>
      <svg width="100%" height="240" viewBox={`0 0 ${W} ${H}`} className="block">
        <defs>
          <pattern id="agents-graph-grid" width="20" height="20" patternUnits="userSpaceOnUse">
            <path d="M 20 0 L 0 0 0 20" fill="none" stroke="rgba(100,116,139,0.07)" strokeWidth="1" />
          </pattern>
        </defs>
        <rect width="100%" height="100%" fill="url(#agents-graph-grid)" />
        {edges.map((e, i) => {
          const s = nodes.find((n) => n.id === e.source);
          const t = nodes.find((n) => n.id === e.target);
          if (!s || !t) return null;
          const isHi = e.source === selectedId || e.target === selectedId;
          return (
            <line
              key={i}
              x1={s.x} y1={s.y} x2={t.x} y2={t.y}
              stroke={isHi ? "rgb(99,102,241)" : "rgba(148,163,184,0.35)"}
              strokeWidth={isHi ? 2 : 1.2}
              strokeDasharray={isHi ? "0" : "4,3"}
            />
          );
        })}
        {nodes.map((n) => {
          const isSel = n.id === selectedId;
          const fill = n.scope === "global"
            ? isSel ? "rgb(245,158,11)" : "rgb(99,102,241)"
            : isSel ? "rgb(245,158,11)" : "rgb(167,139,250)";
          return (
            <g key={n.id} style={{ cursor: "pointer" }} onClick={() => onSelect(n.id)}>
              <circle
                cx={n.x} cy={n.y} r={isSel ? 9 : 6}
                fill={fill}
                stroke={isSel ? "#fff" : "transparent"} strokeWidth={1.5}
              />
              <text
                x={n.x} y={n.y - 12} textAnchor="middle"
                fontSize={10} fontWeight={isSel ? 700 : 400}
                fill="currentColor"
              >
                {n.title.length > 14 ? n.title.slice(0, 13) + "…" : n.title}
              </text>
            </g>
          );
        })}
      </svg>
    </div>
  );
}

// ── Skills / AGENTS.md viewer ───────────────────────────────────────────────
function SkillsViewer() {
  const [files, setFiles] = useState<SkillOrAgentFile[]>([]);
  const [loading, setLoading] = useState(false);
  const [search, setSearch] = useState("");
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [error, setError] = useState("");

  const load = async () => {
    setLoading(true);
    setError("");
    try {
      const list = await ipc.listSkillsAndAgents();
      setFiles(list);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(`Failed to load skills: ${msg}`);
    } finally {
      setLoading(false);
    }
  };
  useEffect(() => { load(); }, []);

  const sources = useMemo(() => {
    const seen = new Set<string>();
    const out: string[] = [];
    for (const f of files) if (!seen.has(f.source)) { seen.add(f.source); out.push(f.source); }
    return out;
  }, [files]);

  const filtered = useMemo(() => {
    const q = search.toLowerCase();
    return files.filter(
      (f) =>
        !q ||
        f.name.toLowerCase().includes(q) ||
        f.path.toLowerCase().includes(q) ||
        f.content.toLowerCase().includes(q)
    );
  }, [files, search]);

  const bySource = useMemo(() => {
    const m = new Map<string, SkillOrAgentFile[]>();
    for (const f of filtered) {
      const arr = m.get(f.source) ?? [];
      arr.push(f);
      m.set(f.source, arr);
    }
    return m;
  }, [filtered]);

  const selected = selectedPath ? files.find((f) => f.path === selectedPath) ?? null : null;

  return (
    <div className="flex gap-3 h-full min-h-0">
      <div className="w-[320px] flex-shrink-0 glass-card p-2 flex flex-col min-h-0">
        <div className="flex items-center gap-1.5 mb-1.5">
          <span className="text-[11px] font-semibold flex-1">Skill & Agent Files</span>
          <Button size="sm" variant="ghost" onClick={load} className="h-6 w-6 p-0">
            <RefreshCw className={cn("h-3 w-3", loading && "animate-spin")} />
          </Button>
        </div>
        <div className="relative mb-1.5">
          <Search className="absolute left-2 top-1/2 -translate-y-1/2 h-3 w-3 text-muted-foreground" />
          <Input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search skills, agents, keywords…"
            className="h-7 text-[11px] pl-7"
          />
        </div>
        {error && (
          <div className="text-[10.5px] text-red-400 border border-red-500/30 bg-red-500/10 rounded p-1.5 mb-1.5">
            {error}
          </div>
        )}
        <ScrollArea className="flex-1 min-h-0">
          {!loading && files.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              <div className="text-3xl mb-2 opacity-50">🗺</div>
              <div className="text-[11px] font-semibold">No skills or agent files found</div>
              <div className="text-[10px] mt-1">
                Click the refresh button to search for SKILL.md and AGENTS.md files.
              </div>
            </div>
          ) : (
            sources.map((src) => {
              const items = bySource.get(src) ?? [];
              if (items.length === 0) return null;
              return (
                <div key={src} className="mb-2">
                  <div className="flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-wider text-muted-foreground px-1 py-1">
                    <span>{src.includes("Gemini") ? "⚡" : src.includes("Plugin") ? "🔌" : "📁"}</span>
                    {src}
                    <span className="ml-auto text-muted-foreground/70 normal-case font-normal">
                      {items.length} file{items.length === 1 ? "" : "s"}
                    </span>
                  </div>
                  {items.map((f) => (
                    <button
                      key={f.path}
                      onClick={() => setSelectedPath(f.path)}
                      className={cn(
                        "w-full text-left flex items-center gap-1.5 px-2 py-1 rounded text-[11px] hover:bg-muted/40",
                        selectedPath === f.path && "bg-muted/60"
                      )}
                    >
                      <span>{f.is_skill ? "🛠" : "🤖"}</span>
                      <span className="truncate flex-1">{f.name}</span>
                    </button>
                  ))}
                </div>
              );
            })
          )}
        </ScrollArea>
      </div>

      <div className="flex-1 min-h-0 glass-card p-3 overflow-y-auto">
        {selected ? (
          <div className="space-y-2">
            <div>
              <h3 className="text-sm font-semibold">Preview: {selected.name}</h3>
              <p className="text-[10px] text-muted-foreground break-all mt-0.5">{selected.path}</p>
            </div>
            <Separator />
            <pre className="text-[11.5px] font-mono leading-relaxed whitespace-pre-wrap break-words bg-muted/30 rounded p-3 max-h-[600px] overflow-y-auto">
              {selected.content}
            </pre>
          </div>
        ) : (
          <div className="h-full flex flex-col items-center justify-center text-muted-foreground text-center">
            <div className="text-2xl mb-2 opacity-40">📄</div>
            <div className="text-[12px] italic max-w-md">
              Select a skill or agent rule file from the tree on the left to preview its content.
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
