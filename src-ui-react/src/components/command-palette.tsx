import { useEffect, useMemo, useState } from "react";
import { Command } from "cmdk";
import { toast } from "sonner";
import {
  MessageSquare,
  Settings2,
  CheckSquare,
  FileText,
  Calendar,
  KanbanSquare,
  LayoutGrid,
  Activity,
  Cpu,
  Gauge,
  HardDrive,
  Download,
  GitCompare,
  Bot,
  Plug,
  Microscope,
  Server,
  Trash2,
  RotateCw,
  Sparkles,
  Search,
} from "lucide-react";
import { useStore } from "@/store/app-store";
import { ipc } from "@/lib/ipc";
import { cn } from "@/lib/utils";

type IconType = React.ComponentType<{ className?: string }>;

interface NavItem {
  id: string;
  label: string;
  icon: IconType;
  keywords: string;
  group: string;
}

const NAV: NavItem[] = [
  { id: "chat",          label: "Chat",            icon: MessageSquare, keywords: "llm talk message conversation",  group: "General" },
  { id: "settings",      label: "Settings",        icon: Settings2,     keywords: "preferences config theme",       group: "General" },
  { id: "todos",         label: "Todos",           icon: CheckSquare,   keywords: "task checklist",                  group: "Productivity" },
  { id: "notes",         label: "Notes",           icon: FileText,      keywords: "markdown document",               group: "Productivity" },
  { id: "calendar",      label: "Calendar",        icon: Calendar,      keywords: "events schedule recurrence",      group: "Productivity" },
  { id: "planner",       label: "Task Planner",    icon: KanbanSquare,  keywords: "kanban board agent",              group: "Productivity" },
  { id: "overview",      label: "Overview",        icon: LayoutGrid,    keywords: "dashboard home metrics",          group: "System" },
  { id: "observability", label: "Observability",   icon: Activity,      keywords: "logs events telemetry",           group: "System" },
  { id: "instances",     label: "Instances",       icon: Cpu,           keywords: "routing targets",                 group: "System" },
  { id: "monitor",       label: "Agent Monitor",   icon: Gauge,         keywords: "agents live running",             group: "System" },
  { id: "library",       label: "Model Library",   icon: HardDrive,     keywords: "gguf files models disk",          group: "Models & Data" },
  { id: "downloader",    label: "Downloader",      icon: Download,      keywords: "huggingface pull fetch",          group: "Models & Data" },
  { id: "compare",       label: "Compare",         icon: GitCompare,    keywords: "benchmark models side by side",   group: "Models & Data" },
  { id: "agents",        label: "Memory & Skills", icon: Bot,           keywords: "agent memory graph episodic gam", group: "Knowledge & Agents" },
  { id: "mcp",           label: "MCP Tools",       icon: Plug,          keywords: "model context protocol servers",  group: "Knowledge & Agents" },
  { id: "deep-research", label: "Deep Research",   icon: Microscope,    keywords: "investigate web search",          group: "Knowledge & Agents" },
];

interface ActionItem {
  id: string;
  label: string;
  icon: IconType;
  keywords: string;
  group: string;
  run: () => void | Promise<void>;
  destructive?: boolean;
}

export function CommandPalette() {
  const [open, setOpen] = useState(false);
  const setActiveTab = useStore((s) => s.setActiveTab);
  const serverOnline = useStore((s) => s.serverOnline);
  const setServerStatus = useStore((s) => s.setServerStatus);
  const clearObsEvents = useStore((s) => s.clearObsEvents);
  const clearServerLogs = useStore((s) => s.clearServerLogs);
  const setChatMessages = useStore((s) => s.setChatMessages);
  const [serverBusy, setServerBusy] = useState(false);

  // Global ⌘K / Ctrl+K to toggle
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setOpen((v) => !v);
      } else if (e.key === "Escape" && open) {
        setOpen(false);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open]);

  const toggleServer = async () => {
    setServerBusy(true);
    try {
      if (serverOnline) {
        await ipc.serverStop();
        setServerStatus(false, []);
        toast.success("Server stopped");
      } else {
        await ipc.serverStart();
        toast.success("Server starting…");
      }
    } catch (e) {
      toast.error("Server toggle failed", { description: String(e) });
    } finally {
      setServerBusy(false);
    }
  };

  const reloadConfig = async () => {
    try {
      const cfg = await ipc.getConfig();
      useStore.getState().setConfig(cfg);
      toast.success("Config reloaded");
    } catch (e) {
      toast.error("Reload failed", { description: String(e) });
    }
  };

  const clearChat = async () => {
    setChatMessages([]);
    try { await ipc.chatSaveHistory([]); } catch {}
    toast.success("Chat cleared");
  };

  const actions: ActionItem[] = useMemo(() => [
    {
      id: "toggle-server",
      label: serverOnline ? "Stop llama-server" : "Start llama-server",
      icon: Server,
      keywords: "llama server start stop online offline",
      group: "Server",
      run: toggleServer,
    },
    {
      id: "reload-config",
      label: "Reload configuration",
      icon: RotateCw,
      keywords: "refresh config rust",
      group: "Server",
      run: reloadConfig,
    },
    {
      id: "clear-events",
      label: "Clear observability events",
      icon: Trash2,
      keywords: "wipe reset observability telemetry",
      group: "Cleanup",
      run: clearObsEvents,
      destructive: true,
    },
    {
      id: "clear-logs",
      label: "Clear server logs",
      icon: Trash2,
      keywords: "wipe reset logs",
      group: "Cleanup",
      run: clearServerLogs,
      destructive: true,
    },
    {
      id: "clear-chat",
      label: "Clear chat history",
      icon: Sparkles,
      keywords: "wipe reset messages conversation",
      group: "Cleanup",
      run: clearChat,
      destructive: true,
    },
  ], [serverOnline, serverBusy]);

  const handleSelect = (fn: () => void | Promise<void>) => {
    setOpen(false);
    Promise.resolve(fn()).catch(() => {});
  };

  return (
    <Command.Dialog
      open={open}
      onOpenChange={setOpen}
      label="Global command palette"
      className="fixed inset-0 z-50"
      shouldFilter
    >
      <div
        className="fixed inset-0 bg-black/60 backdrop-blur-sm"
        onClick={() => setOpen(false)}
      />
      <div className="fixed left-1/2 top-[18%] z-10 w-[min(640px,92vw)] -translate-x-1/2 glass-card overflow-hidden rounded-xl border border-border/60 shadow-2xl">
        <div className="flex items-center gap-2 border-b border-border/50 px-4 py-3">
          <Search className="h-4 w-4 text-muted-foreground" />
          <Command.Input
            autoFocus
            placeholder="Type a command, search tabs, or jump anywhere…"
            className="flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground/60"
          />
          <kbd className="rounded border border-border/60 bg-background/40 px-1.5 py-0.5 text-[10px] text-muted-foreground">esc</kbd>
        </div>
        <Command.List className="max-h-[60vh] overflow-y-auto p-2">
          <Command.Empty className="py-8 text-center text-sm text-muted-foreground">
            No results found.
          </Command.Empty>

          {(["General", "Productivity", "System", "Models & Data", "Knowledge & Agents"] as const).map((group) => {
            const items = NAV.filter((n) => n.group === group);
            if (items.length === 0) return null;
            return (
              <Command.Group
                key={group}
                heading={group}
                className="[&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:pb-1 [&_[cmdk-group-heading]]:pt-2 [&_[cmdk-group-heading]]:text-[10px] [&_[cmdk-group-heading]]:font-semibold [&_[cmdk-group-heading]]:uppercase [&_[cmdk-group-heading]]:tracking-widest [&_[cmdk-group-heading]]:text-muted-foreground/60"
              >
                {items.map((item) => {
                  const Icon = item.icon;
                  return (
                    <Command.Item
                      key={item.id}
                      value={`${item.label} ${item.keywords} ${item.id}`}
                      onSelect={() => handleSelect(() => setActiveTab(item.id))}
                      className="flex cursor-pointer items-center gap-2.5 rounded-md px-2 py-1.5 text-sm aria-selected:bg-primary/15 aria-selected:text-foreground data-[selected=true]:bg-primary/15"
                    >
                      <Icon className="h-4 w-4 text-muted-foreground" />
                      <span className="flex-1">{item.label}</span>
                      <span className="text-[10px] text-muted-foreground/60">Go to</span>
                    </Command.Item>
                  );
                })}
              </Command.Group>
            );
          })}

          {(["Server", "Cleanup"] as const).map((group) => {
            const items = actions.filter((a) => a.group === group);
            if (items.length === 0) return null;
            return (
              <Command.Group
                key={group}
                heading={group}
                className="[&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:pb-1 [&_[cmdk-group-heading]]:pt-2 [&_[cmdk-group-heading]]:text-[10px] [&_[cmdk-group-heading]]:font-semibold [&_[cmdk-group-heading]]:uppercase [&_[cmdk-group-heading]]:tracking-widest [&_[cmdk-group-heading]]:text-muted-foreground/60"
              >
                {items.map((item) => {
                  const Icon = item.icon;
                  return (
                    <Command.Item
                      key={item.id}
                      value={`${item.label} ${item.keywords} ${item.id}`}
                      onSelect={() => handleSelect(item.run)}
                      className={cn(
                        "flex cursor-pointer items-center gap-2.5 rounded-md px-2 py-1.5 text-sm aria-selected:bg-primary/15 aria-selected:text-foreground data-[selected=true]:bg-primary/15",
                        item.destructive && "text-muted-foreground"
                      )}
                    >
                      <Icon className={cn("h-4 w-4", item.destructive ? "text-destructive/80" : "text-muted-foreground")} />
                      <span className="flex-1">{item.label}</span>
                    </Command.Item>
                  );
                })}
              </Command.Group>
            );
          })}
        </Command.List>
        <div className="flex items-center justify-between border-t border-border/50 bg-background/30 px-3 py-2 text-[10px] text-muted-foreground/70">
          <div className="flex items-center gap-3">
            <span className="flex items-center gap-1">
              <kbd className="rounded border border-border/60 bg-background/40 px-1 py-0.5">↑</kbd>
              <kbd className="rounded border border-border/60 bg-background/40 px-1 py-0.5">↓</kbd>
              navigate
            </span>
            <span className="flex items-center gap-1">
              <kbd className="rounded border border-border/60 bg-background/40 px-1 py-0.5">↵</kbd>
              select
            </span>
          </div>
          <span className="flex items-center gap-1">
            <kbd className="rounded border border-border/60 bg-background/40 px-1 py-0.5">⌘K</kbd>
            toggle
          </span>
        </div>
      </div>
    </Command.Dialog>
  );
}
