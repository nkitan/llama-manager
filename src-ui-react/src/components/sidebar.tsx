import {
  MessageSquare,
  Settings2,
  CheckSquare,
  FileText,
  Calendar,
  KanbanSquare,
  LayoutGrid,
  Activity,
  Server,
  ChevronRight,
} from "lucide-react";
import { useStore } from "@/store/app-store";
import { cn } from "@/lib/utils";
import { ipc } from "@/lib/ipc";
import { useState } from "react";

interface NavItem {
  id: string;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
}

const NAV_GROUPS: { label: string; items: NavItem[] }[] = [
  {
    label: "General",
    items: [
      { id: "chat", label: "Chat", icon: MessageSquare },
      { id: "settings", label: "Settings", icon: Settings2 },
    ],
  },
  {
    label: "Productivity",
    items: [
      { id: "todos", label: "Todos", icon: CheckSquare },
      { id: "notes", label: "Notes", icon: FileText },
      { id: "calendar", label: "Calendar", icon: Calendar },
      { id: "planner", label: "Task Planner", icon: KanbanSquare },
    ],
  },
  {
    label: "System",
    items: [
      { id: "overview", label: "Overview", icon: LayoutGrid },
      { id: "observability", label: "Observability", icon: Activity },
    ],
  },
];

export function Sidebar() {
  const { activeTab, setActiveTab, config, serverOnline, serverModels, setServerStatus } =
    useStore();
  const [serverLoading, setServerLoading] = useState(false);

  const toggleServer = async () => {
    if (!config) return;
    setServerLoading(true);
    try {
      if (serverOnline) {
        await ipc.serverStop();
        setServerStatus(false, []);
      } else {
        await ipc.serverStart();
      }
    } catch (e) {
      console.error("server toggle:", e);
    } finally {
      setServerLoading(false);
    }
  };

  return (
    <div className="glass-sidebar flex w-52 flex-col flex-shrink-0 py-2 overflow-hidden">
      {/* Nav groups */}
      <div className="flex-1 overflow-y-auto px-2 space-y-4">
        {NAV_GROUPS.map((group) => (
          <div key={group.label}>
            <p className="px-2 pb-1 text-[10px] font-semibold uppercase tracking-widest text-muted-foreground/60">
              {group.label}
            </p>
            <div className="space-y-0.5">
              {group.items.map((item) => (
                <NavButton
                  key={item.id}
                  item={item}
                  active={activeTab === item.id}
                  onClick={() => setActiveTab(item.id)}
                />
              ))}
            </div>
          </div>
        ))}
      </div>

      {/* Server status footer */}
      <div className="mx-2 mt-2 rounded-md border border-border/40 p-2">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-1.5 min-w-0">
            <div
              className={cn(
                "h-2 w-2 rounded-full flex-shrink-0 transition-colors",
                serverOnline ? "bg-emerald-400 shadow-[0_0_6px_rgba(52,211,153,0.6)]" : "bg-muted-foreground/40"
              )}
            />
            <span className="text-xs text-muted-foreground truncate">
              {serverOnline
                ? serverModels[0]?.split("/").pop()?.replace(/\.gguf$/i, "") ?? "Online"
                : "Server offline"}
            </span>
          </div>
          <button
            onClick={toggleServer}
            disabled={serverLoading || !config}
            className={cn(
              "flex h-5 w-5 items-center justify-center rounded text-muted-foreground transition-colors flex-shrink-0",
              serverOnline ? "hover:text-destructive" : "hover:text-emerald-400"
            )}
            title={serverOnline ? "Stop server" : "Start server"}
          >
            {serverLoading ? (
              <div className="h-3 w-3 rounded-full border border-current border-t-transparent animate-spin" />
            ) : (
              <Server className="h-3 w-3" />
            )}
          </button>
        </div>
        {serverOnline && config && (
          <p className="mt-1 text-[10px] text-muted-foreground/60 truncate">
            {config.host}:{config.port}
          </p>
        )}
      </div>
    </div>
  );
}

function NavButton({
  item,
  active,
  onClick,
}: {
  item: NavItem;
  active: boolean;
  onClick: () => void;
}) {
  const Icon = item.icon;
  return (
    <button
      onClick={onClick}
      className={cn(
        "flex w-full items-center gap-2.5 rounded-md px-2 py-1.5 text-sm transition-colors no-drag text-left",
        active
          ? "bg-primary/15 text-foreground font-medium"
          : "text-muted-foreground hover:bg-accent hover:text-foreground"
      )}
    >
      <Icon className="h-4 w-4 flex-shrink-0" />
      <span className="truncate">{item.label}</span>
      {active && <ChevronRight className="ml-auto h-3 w-3 opacity-50" />}
    </button>
  );
}
