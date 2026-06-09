import { useEffect, useMemo, useState, useCallback } from "react";
import {
  Plus, Trash2, Save, X, RefreshCw, Bot, Network, FileCode2,
  Sparkles, Tag, Globe, FolderOpen, Search, Folder, FileText,
  ChevronRight, ChevronDown, Activity, Brain, Maximize2, Minimize2,
  PanelRightOpen, Copy,
} from "lucide-react";
import {
  ReactFlow, ReactFlowProvider, Background, Controls, MiniMap,
  Handle, Position, type Node, type Edge, type NodeProps,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { ipc, type Memory, type SkillOrAgentFile } from "@/lib/ipc";
import { useStore } from "@/store/app-store";
import { resolveTarget } from "@/lib/target";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Switch } from "@/components/ui/switch";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
} from "@/components/ui/drawer";
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
  const [showGraph, setShowGraph] = useState(false);
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
            size="sm" variant={showGraph ? "secondary" : "outline"}
            onClick={() => setShowGraph((v) => !v)}
            className="h-7 text-[11px] w-full"
          >
            {showGraph ? <FolderOpen className="h-3 w-3" /> : <Brain className="h-3 w-3" />}
            {showGraph ? "Workspace" : "Graph View"}
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
        {showParallelWarn && !showGraph && (
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

        {showGraph ? (
          <GamMemorySimulation />
        ) : (
          <>
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
          </>
        )}
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
  const [drawerOpen, setDrawerOpen] = useState(false);

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
                <Button size="sm" variant="ghost" onClick={() => setDrawerOpen(true)} className="h-6 text-[10px]">
                  <PanelRightOpen className="h-2.5 w-2.5" /> Pop Out
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

      <Drawer open={drawerOpen} onOpenChange={setDrawerOpen}>
        <DrawerContent>
          <DrawerHeader>
            <DrawerTitle>Agent Copilot — full output</DrawerTitle>
            <DrawerDescription>
              Live response from the agent. Append to the note, copy, or close.
            </DrawerDescription>
          </DrawerHeader>
          <div className="flex-1 min-h-0 px-3 pb-2">
            <div className="h-[60vh] rounded-md border border-border/40 bg-black/40 p-3 overflow-auto">
              <pre className="text-[11px] font-mono text-emerald-200/90 whitespace-pre-wrap break-words leading-relaxed">
                {agentOutput}
              </pre>
            </div>
          </div>
          <div className="px-3 pb-4 flex gap-2">
            <Button
              size="sm"
              variant="outline"
              onClick={appendOutput}
              className="flex-1 h-8 text-xs"
            >
              <Plus className="h-3 w-3" /> Append to Note
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => {
                navigator.clipboard.writeText(agentOutput).catch(() => {});
              }}
              className="h-8 text-xs"
            >
              <Copy className="h-3 w-3" /> Copy
            </Button>
          </div>
        </DrawerContent>
      </Drawer>
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
  const [expanded, setExpanded] = useState(false);

  const flowNodes: Node[] = useMemo(() => {
    if (memories.length === 0) return [];
    const total = memories.length;
    const r = 200;
    return memories.map((m, i) => {
      const angle = (i / total) * 2 * Math.PI;
      const isSel = m.id === selectedId;
      const base = m.scope === "global" ? "rgb(99,102,241)" : "rgb(167,139,250)";
      return {
        id: m.id,
        position: { x: 320 + r * Math.cos(angle), y: 200 + r * Math.sin(angle) },
        data: {
          label: m.title || "Untitled",
          scope: m.scope,
          selected: isSel,
          color: isSel ? "rgb(245,158,11)" : base,
        },
        type: "memory",
      };
    });
  }, [memories, selectedId]);

  const flowEdges: Edge[] = useMemo(() => {
    const ids = new Set(memories.map((m) => m.id));
    const out: Edge[] = [];
    for (const m of memories) {
      for (const l of m.links) {
        if (ids.has(l) && ids.has(m.id)) {
          const isHi = m.id === selectedId || l === selectedId;
          out.push({
            id: `${m.id}-${l}`,
            source: m.id,
            target: l,
            animated: isHi,
            style: {
              stroke: isHi ? "rgb(99,102,241)" : "rgba(148,163,184,0.4)",
              strokeWidth: isHi ? 2 : 1.2,
              strokeDasharray: isHi ? "0" : "4,3",
            },
          });
        }
      }
    }
    return out;
  }, [memories, selectedId]);

  if (memories.length === 0) return null;

  return (
    <div
      className={cn(
        "glass-card flex flex-col flex-shrink-0 transition-all",
        expanded ? "fixed inset-3 z-40" : "h-[300px]"
      )}
    >
      <div className="flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-wider text-muted-foreground p-2 border-b border-border/40 flex-shrink-0">
        <Network className="h-3 w-3" />
        <span className="flex-1">Knowledge Graph</span>
        <span className="text-muted-foreground/70 normal-case font-normal">
          {memories.length} nodes · {flowEdges.length} links
        </span>
        <Button
          size="sm" variant="ghost"
          onClick={() => setExpanded((v) => !v)}
          className="h-5 w-5 p-0"
        >
          {expanded ? <Minimize2 className="h-3 w-3" /> : <Maximize2 className="h-3 w-3" />}
        </Button>
      </div>
      <div className="flex-1 min-h-0">
        <ReactFlowProvider>
          <ReactFlow
            nodes={flowNodes}
            edges={flowEdges}
            nodeTypes={MEMORY_NODE_TYPES}
            onNodeClick={(_e, n) => onSelect(n.id)}
            fitView
            fitViewOptions={{ padding: 0.2 }}
            minZoom={0.2}
            maxZoom={2}
            proOptions={{ hideAttribution: true }}
            defaultEdgeOptions={{ type: "default" }}
            style={{ background: "transparent" }}
          >
            <Background gap={20} color="rgba(100,116,139,0.15)" />
            <Controls
              position="bottom-right"
              showInteractive={false}
              className="!bg-card/80 !border !border-border/40 !rounded-md"
            />
            <MiniMap
              position="bottom-left"
              pannable
              zoomable
              maskColor="rgba(0,0,0,0.5)"
              nodeColor={(n) => (n.data as { color?: string })?.color ?? "rgb(99,102,241)"}
              style={{
                background: "hsl(var(--card) / 0.6)",
                border: "1px solid hsl(var(--border) / 0.4)",
                borderRadius: "6px",
              }}
            />
          </ReactFlow>
        </ReactFlowProvider>
      </div>
    </div>
  );
}

type MemoryNodeData = { label: string; scope: string; selected: boolean; color: string };

const MemoryNode = ({ data, selected }: NodeProps<Node<MemoryNodeData>>) => {
  return (
    <div
      className="px-2 py-1 rounded-md text-[10.5px] font-semibold border-2 transition-shadow"
      style={{
        background: "hsl(var(--card) / 0.95)",
        color: data.color,
        borderColor: data.color,
        boxShadow: selected
          ? `0 0 0 2px hsl(var(--background)), 0 0 12px ${data.color}`
          : "none",
        minWidth: 60,
        textAlign: "center",
        maxWidth: 140,
        overflow: "hidden",
        textOverflow: "ellipsis",
        whiteSpace: "nowrap",
      }}
      title={data.label}
    >
      <Handle type="target" position={Position.Top} style={{ opacity: 0, pointerEvents: "none" }} />
      {data.label}
      <Handle type="source" position={Position.Bottom} style={{ opacity: 0, pointerEvents: "none" }} />
    </div>
  );
};

const MEMORY_NODE_TYPES = { memory: MemoryNode };

// ── GAM episodic memory simulation ──────────────────────────────────────────
interface EpisodicLog { text: string; timestamp: string }
interface SimNode { id: string; label: string; x: number; y: number }
interface SimEdge { source: string; target: string }

function GamMemorySimulation() {
  const [episodicBuffer, setEpisodicBuffer] = useState<EpisodicLog[]>([
    { text: "Session initiated: connected to model server", timestamp: "23:15:00" },
    { text: "Executed search for local memory context", timestamp: "23:15:05" },
    { text: "User queried: 'Optimize CUDA configurations'", timestamp: "23:15:10" },
  ]);
  const [divergenceThreshold] = useState(0.4);
  const [divergenceScore, setDivergenceScore] = useState(0.15);
  const [newLogInput, setNewLogInput] = useState("");
  const [animConsolidating, setAnimConsolidating] = useState(false);
  const [semanticNodes, setSemanticNodes] = useState<SimNode[]>([
    { id: "project", label: "Project Core", x: 380, y: 280 },
    { id: "user_prefs", label: "User Preferences", x: 180, y: 140 },
    { id: "tech_stack", label: "Tech Stack (Rust/Tauri)", x: 580, y: 140 },
  ]);
  const [semanticEdges, setSemanticEdges] = useState<SimEdge[]>([
    { source: "user_prefs", target: "project" },
    { source: "project", target: "tech_stack" },
  ]);

  const addLog = () => {
    const text = newLogInput.trim();
    if (!text) return;
    const now = new Date();
    const ts = `${String(now.getHours()).padStart(2, "0")}:${String(now.getMinutes()).padStart(2, "0")}:${String(now.getSeconds()).padStart(2, "0")}`;
    setEpisodicBuffer((buf) => [...buf, { text, timestamp: ts }]);
    setDivergenceScore((s) => Math.min(1, s + 0.12));
    setNewLogInput("");
  };

  const triggerConsolidation = () => {
    if (episodicBuffer.length === 0) return;
    setAnimConsolidating(true);
    const last = episodicBuffer[episodicBuffer.length - 1];
    const t = last.text.trim();
    const clean = t.length > 22 ? `${t.slice(0, 19)}…` : t;
    let nodeLabel: string;
    if (clean.toLowerCase().includes("cuda")) nodeLabel = "CUDA Optimization";
    else if (clean.toLowerCase().includes("search")) nodeLabel = "Search Context";
    else nodeLabel = clean;

    const nodeId = `node_${Date.now()}`;
    const nodeCount = semanticNodes.length;
    const angle = (nodeCount * 2 * Math.PI) / 6;
    const radius = 160;
    const x = 380 + radius * Math.cos(angle);
    const y = 280 + radius * Math.sin(angle);

    setSemanticNodes((nodes) => [...nodes, { id: nodeId, label: nodeLabel, x, y }]);
    setSemanticEdges((edges) => [...edges, { source: "project", target: nodeId }]);
    setEpisodicBuffer([]);
    setDivergenceScore(0.05);

    window.setTimeout(() => setAnimConsolidating(false), 800);
  };

  const scoreColor =
    divergenceScore >= divergenceThreshold
      ? "#ef4444"
      : divergenceScore >= divergenceThreshold * 0.7
        ? "#f59e0b"
        : "#10b981";

  return (
    <div className="flex gap-3 flex-wrap w-full items-stretch flex-1 min-h-0">
      <div className="flex-[1.2] min-w-[320px] flex flex-col gap-3 min-h-0">
        <div className="glass-card p-3 flex flex-col gap-2 flex-1 min-h-0">
          <div className="flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-wider text-muted-foreground">
            <Activity className="h-3 w-3" />
            📋 Episodic Buffer (Event Graph)
          </div>
          <ScrollArea className="flex-1 min-h-[120px] max-h-[200px]">
            {episodicBuffer.length === 0 ? (
              <p className="text-center text-[11px] text-muted-foreground italic py-6">
                Episodic buffer is clean.
              </p>
            ) : (
              <div className="space-y-1.5 pr-1">
                {episodicBuffer.map((log, i) => (
                  <div
                    key={i}
                    className="flex gap-2 px-2.5 py-1.5 border-l-2 border-accent bg-muted/30 rounded-r text-[11.5px]"
                  >
                    <span className="font-mono opacity-60 text-[10.5px]">[{log.timestamp}]</span>
                    <span>{log.text}</span>
                  </div>
                ))}
              </div>
            )}
          </ScrollArea>
          <div className="bg-muted/30 border border-border/40 rounded-md p-2.5">
            <div className="flex items-center justify-between text-[11px] font-semibold mb-1.5">
              <span>Semantic Divergence Score</span>
              <span
                className="font-bold"
                style={{ color: divergenceScore >= divergenceThreshold ? "#ef4444" : "hsl(var(--accent))" }}
              >
                {divergenceScore.toFixed(2)} / {divergenceThreshold.toFixed(2)}
              </span>
            </div>
            <div className="w-full h-2 bg-black/10 dark:bg-white/10 rounded-full overflow-hidden">
              <div
                className="h-full transition-all duration-300 ease-out"
                style={{
                  width: `${Math.min(100, divergenceScore * 100)}%`,
                  background: scoreColor,
                }}
              />
            </div>
          </div>
          <div className="flex flex-col gap-2">
            <div className="flex gap-2">
              <Input
                value={newLogInput}
                onChange={(e) => setNewLogInput(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && addLog()}
                placeholder="Add simulated agent event…"
                className="flex-1 h-7 text-[12px]"
              />
              <Button size="sm" onClick={addLog} className="h-7 text-[11px]">Add</Button>
            </div>
            <Button
              onClick={triggerConsolidation}
              disabled={episodicBuffer.length === 0 || animConsolidating}
              className="h-8 font-bold"
            >
              {animConsolidating ? "Consolidating…" : "🧬 Trigger Consolidation"}
            </Button>
          </div>
        </div>
      </div>

      <div className="flex-[1.8] min-w-[400px] glass-card p-3 flex flex-col min-h-0">
        <div className="flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-wider text-muted-foreground mb-1.5">
          <Network className="h-3 w-3" />
          🕸️ Topic Associative Graph
        </div>
        <div className="flex-1 min-h-[380px] rounded-lg overflow-hidden border border-border/40 bg-muted/20">
          <ReactFlowProvider>
            <ReactFlow
              nodes={semanticNodes.map((n) => ({
                id: n.id,
                position: { x: n.x, y: n.y },
                data: {
                  label: n.label,
                  isCore: n.id === "project",
                  pulse: animConsolidating,
                },
                type: "topic",
              }))}
              edges={semanticEdges.map((e) => ({
                id: `${e.source}-${e.target}`,
                source: e.source,
                target: e.target,
                style: { stroke: "rgba(99,102,241,0.4)", strokeWidth: 2, strokeDasharray: "4,4" },
              }))}
              nodeTypes={TOPIC_NODE_TYPES}
              fitView
              fitViewOptions={{ padding: 0.2 }}
              minZoom={0.5}
              maxZoom={1.5}
              nodesDraggable={false}
              nodesConnectable={false}
              elementsSelectable={false}
              proOptions={{ hideAttribution: true }}
              style={{ background: "transparent" }}
            >
              <Background gap={20} color="rgba(100,116,139,0.1)" />
            </ReactFlow>
          </ReactFlowProvider>
        </div>
      </div>
    </div>
  );
}

type TopicNodeData = { label: string; isCore: boolean; pulse: boolean };
const TopicNode = ({ data }: NodeProps<Node<TopicNodeData>>) => {
  return (
    <div
      className={cn(
        "px-2 py-1 rounded-full text-[10.5px] font-semibold border-2",
        data.pulse && "animate-pulse"
      )}
      style={{
        background: data.isCore ? "hsl(var(--primary))" : "rgb(16,185,241)",
        color: data.isCore ? "hsl(var(--primary-foreground))" : "#0c0a09",
        borderColor: "#fff",
        boxShadow: "0 0 8px rgba(0,0,0,0.4)",
        whiteSpace: "nowrap",
      }}
    >
      <Handle type="target" position={Position.Top} style={{ opacity: 0, pointerEvents: "none" }} />
      {data.label}
      <Handle type="source" position={Position.Bottom} style={{ opacity: 0, pointerEvents: "none" }} />
    </div>
  );
};
const TOPIC_NODE_TYPES = { topic: TopicNode };

// ── Skills / AGENTS.md viewer ───────────────────────────────────────────────
interface TreeItem {
  name: string;
  full_path: string | null;
  is_skill: boolean;
  content: string;
  children: TreeItem[];
}

function getRelativePath(path: string, source: string): string {
  const prefix =
    source === "Workspace" ? "/home/notroot/Work/llama-manager"
    : source === "Gemini Skills" ? "/home/notroot/.gemini/skills"
    : source === "Gemini Plugins" ? "/home/notroot/.gemini/config/plugins"
    : "";
  if (prefix && path.startsWith(prefix)) {
    return path.slice(prefix.length).replace(/^\/+/, "");
  }
  return path;
}

function insertIntoTree(
  tree: TreeItem[],
  segments: string[],
  fullPath: string,
  isSkill: boolean,
  content: string
): void {
  if (segments.length === 0) return;
  const [head, ...rest] = segments;
  if (rest.length === 0) {
    if (!tree.some((it) => it.name === head && it.full_path !== null)) {
      tree.push({
        name: head,
        full_path: fullPath,
        is_skill: isSkill,
        content,
        children: [],
      });
    }
    return;
  }
  let dirIdx = tree.findIndex((it) => it.name === head && it.full_path === null);
  if (dirIdx === -1) {
    tree.push({ name: head, full_path: null, is_skill: false, content: "", children: [] });
    dirIdx = tree.length - 1;
  }
  insertIntoTree(tree[dirIdx].children, rest, fullPath, isSkill, content);
}

function sortTree(tree: TreeItem[]): void {
  tree.sort((a, b) => {
    const aIsDir = a.full_path === null;
    const bIsDir = b.full_path === null;
    if (aIsDir !== bIsDir) return aIsDir ? -1 : 1;
    return a.name.localeCompare(b.name);
  });
  for (const item of tree) sortTree(item.children);
}

function SkillsViewer() {
  const [files, setFiles] = useState<SkillOrAgentFile[]>([]);
  const [loading, setLoading] = useState(false);
  const [search, setSearch] = useState("");
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [error, setError] = useState("");
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

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

  const treesBySource = useMemo(() => {
    const m = new Map<string, TreeItem[]>();
    for (const src of sources) {
      const items = filtered.filter((f) => f.source === src);
      if (items.length === 0) continue;
      const tree: TreeItem[] = [];
      for (const f of items) {
        const rel = getRelativePath(f.path, f.source);
        const segments = rel.split("/").filter(Boolean);
        insertIntoTree(tree, segments, f.path, f.is_skill, f.content);
      }
      sortTree(tree);
      m.set(src, tree);
    }
    return m;
  }, [filtered, sources]);

  const selected = selectedPath ? files.find((f) => f.path === selectedPath) ?? null : null;

  const toggleExpanded = (key: string) => {
    setExpanded((s) => {
      const next = new Set(s);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  };

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
            Array.from(treesBySource.entries()).map(([src, tree]) => (
              <div key={src} className="mb-2 rounded-md border border-border/40 bg-card/40 overflow-hidden">
                <div className="flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-wider text-muted-foreground px-2 py-1.5 bg-muted/40 border-b border-border/40">
                  <span>{src.includes("Gemini") ? "⚡" : src.includes("Plugin") ? "🔌" : "📁"}</span>
                  <span className="flex-1">{src}</span>
                  <span className="text-muted-foreground/70 normal-case font-normal">
                    {tree.length} item{tree.length === 1 ? "" : "s"}
                  </span>
                </div>
                <div className="py-1">
                  {tree.map((item) => (
                    <TreeNode
                      key={item.name}
                      item={item}
                      depth={0}
                      sourceKey={src}
                      expanded={expanded}
                      onToggle={toggleExpanded}
                      selectedPath={selectedPath}
                      onSelect={setSelectedPath}
                    />
                  ))}
                </div>
              </div>
            ))
          )}
        </ScrollArea>
      </div>

      <div className="flex-1 min-h-0 glass-card p-3 overflow-y-auto">
        {selected ? (
          <div className="space-y-2">
            <div className="flex items-center gap-1.5">
              <span className="text-[9px] font-bold uppercase px-1.5 py-0.5 rounded bg-primary/20 text-primary border border-primary/30">
                {selected.is_skill ? "SKILL" : "AGENTS"}
              </span>
              <h3 className="text-sm font-semibold">Preview: {selected.name}</h3>
            </div>
            <p className="text-[10px] text-muted-foreground break-all">{selected.path}</p>
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

function TreeNode({
  item, depth, sourceKey, expanded, onToggle, selectedPath, onSelect,
}: {
  item: TreeItem;
  depth: number;
  sourceKey: string;
  expanded: Set<string>;
  onToggle: (key: string) => void;
  selectedPath: string | null;
  onSelect: (path: string) => void;
}) {
  const isDir = item.full_path === null;
  const key = `${sourceKey}/${item.name}-${depth}`;
  const isExpanded = expanded.has(key);

  if (isDir) {
    return (
      <div>
        <button
          onClick={() => onToggle(key)}
          className="w-full text-left flex items-center gap-1.5 py-1 hover:bg-muted/30 transition-colors"
          style={{ paddingLeft: `${depth * 12 + 8}px`, paddingRight: "8px" }}
        >
          {isExpanded ? (
            <ChevronDown className="h-3 w-3 text-muted-foreground flex-shrink-0" />
          ) : (
            <ChevronRight className="h-3 w-3 text-muted-foreground flex-shrink-0" />
          )}
          <Folder className="h-3 w-3 text-muted-foreground flex-shrink-0" />
          <span className="text-[11.5px] font-semibold truncate">{item.name}</span>
        </button>
        {isExpanded && (
          <div>
            {item.children.map((child, i) => (
              <TreeNode
                key={`${child.name}-${i}`}
                item={child}
                depth={depth + 1}
                sourceKey={sourceKey}
                expanded={expanded}
                onToggle={onToggle}
                selectedPath={selectedPath}
                onSelect={onSelect}
              />
            ))}
          </div>
        )}
      </div>
    );
  }

  const isSelected = selectedPath === item.full_path;
  return (
    <button
      onClick={() => onSelect(item.full_path!)}
      className={cn(
        "w-full text-left flex items-center gap-1.5 py-1 border-b border-border/20 transition-colors",
        isSelected ? "bg-primary/10" : "hover:bg-muted/30"
      )}
      style={{ paddingLeft: `${depth * 12 + 8}px`, paddingRight: "8px" }}
    >
      <span className="w-3 flex-shrink-0" />
      <FileText className="h-3 w-3 text-muted-foreground flex-shrink-0" />
      <span className="text-[11.5px] truncate flex-1 min-w-0">{item.name}</span>
      <span
        className={cn(
          "text-[9px] font-bold uppercase px-1.5 py-0.5 rounded-full",
          item.is_skill
            ? "bg-indigo-500/20 text-indigo-400 border border-indigo-500/40"
            : "bg-amber-500/20 text-amber-400 border border-amber-500/40"
        )}
      >
        {item.is_skill ? "SKILL" : "AGENTS"}
      </span>
    </button>
  );
}
