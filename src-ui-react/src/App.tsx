import { useEffect, useRef } from "react";
import { Titlebar } from "@/components/titlebar";
import { Sidebar } from "@/components/sidebar";
import { useStore } from "@/store/app-store";
import { ipc, onChatEvent, onAgentEvent, onConfigChanged, onServerLog } from "@/lib/ipc";
import { applyTheme } from "@/lib/theme";
import { TooltipProvider } from "@/components/ui/tooltip";

// Lazy tab imports
import { ChatTab } from "@/tabs/chat";
import { SettingsTab } from "@/tabs/settings";
import { OverviewTab } from "@/tabs/overview";
import { ObservabilityTab } from "@/tabs/observability";
import { NotesTab } from "@/tabs/notes";
import { TodosTab } from "@/tabs/todos";
import { CalendarTab } from "@/tabs/calendar";
import { PlannerTab } from "@/tabs/planner";
import { LibraryTab } from "@/tabs/library";
import { DownloaderTab } from "@/tabs/downloader";
import { InstancesTab } from "@/tabs/instances";
import { MonitorTab } from "@/tabs/monitor";
import { AgentsTab } from "@/tabs/agents";
import { McpTab } from "@/tabs/mcp";
import { CompareTab } from "@/tabs/compare";
import { DeepResearchTab } from "@/tabs/deep-research";

export default function App() {
  const {
    activeTab,
    setConfig,
    setServerStatus,
    setChatGenerating,
    setChatStreamId,
    setChatError,
    updateLastAssistant,
    addChatMessage,
    chatStreamId,
    setAgentPlan,
    updatePlanStep,
    addAgentTrace,
    addApproval,
    setActiveAgentId,
    addObsEvent,
    addServerLog,
  } = useStore();

  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const configRef = useRef(useStore.getState().config);

  // ── Bootstrap ─────────────────────────────────────────────────────────────
  useEffect(() => {
    ipc.getConfig().then((cfg) => {
      setConfig(cfg);
      configRef.current = cfg;
      applyTheme(cfg);
    });

    ipc.chatLoadHistory().then((history) => {
      if (history.length > 0) useStore.getState().setChatMessages(history);
    });
  }, []);

  // ── Server status poll (every 3s) ──────────────────────────────────────────
  useEffect(() => {
    const poll = async () => {
      const cfg = useStore.getState().config;
      if (!cfg) return;
      try {
        const [status, models] = await Promise.all([
          ipc.serverStatus(),
          ipc.chatListModels(cfg.host, cfg.port),
        ]);
        setServerStatus(status || models.online, models.models);
      } catch {
        setServerStatus(false, []);
      }
    };
    poll();
    pollRef.current = setInterval(poll, 3000);
    return () => { if (pollRef.current) clearInterval(pollRef.current); };
  }, []);

  // ── IPC event listeners (registered once, survive tab switches) ────────────
  useEffect(() => {
    const unlisteners: Array<() => void> = [];

    onChatEvent((ev) => {
      const sid = useStore.getState().chatStreamId;
      if (ev.kind === "token") {
        if (ev.stream_id === sid) updateLastAssistant(ev.delta);
        addObsEvent({ ts: Date.now(), kind: "chat:token", id: ev.stream_id, content: ev.delta });
      } else if (ev.kind === "done") {
        setChatGenerating(false);
        setChatStreamId(null);
        ipc.chatSaveHistory(useStore.getState().chatMessages).catch(() => {});
        addObsEvent({ ts: Date.now(), kind: "chat:done", id: ev.stream_id, content: "" });
      } else if (ev.kind === "error") {
        setChatGenerating(false);
        setChatStreamId(null);
        setChatError(ev.message);
        addObsEvent({ ts: Date.now(), kind: "chat:error", id: ev.stream_id, content: ev.message });
      }
    }).then((fn) => unlisteners.push(fn));

    onAgentEvent((ev) => {
      addObsEvent({ ts: Date.now(), kind: `agent:${ev.kind}`, id: ev.agent_id, content: "content" in ev ? String((ev as { content?: unknown }).content ?? "") : "" });
      if (ev.kind === "started") {
        setActiveAgentId(ev.agent_id);
        addAgentTrace(`[${ev.role}] ${ev.task}`);
      } else if (ev.kind === "plan") {
        setAgentPlan(ev.steps);
      } else if (ev.kind === "plan_update") {
        updatePlanStep(ev.step_id, ev.status as "pending" | "running" | "done" | "failed");
      } else if (ev.kind === "step") {
        addAgentTrace(`💭 ${ev.thought}`);
      } else if (ev.kind === "token") {
        updateLastAssistant(ev.delta);
      } else if (ev.kind === "tool_call") {
        addAgentTrace(`🔧 ${ev.call.tool}(${JSON.stringify(ev.call.args).slice(0, 80)})`);
        addObsEvent({ ts: Date.now(), kind: "agent:tool_call", id: ev.call.tool, content: JSON.stringify(ev.call.args) });
      } else if (ev.kind === "tool_result") {
        addAgentTrace(`✓ ${ev.result.ok ? "ok" : "err"}: ${ev.result.output.slice(0, 100)}`);
        addObsEvent({ ts: Date.now(), kind: "agent:tool_ok", id: ev.result.call_id, content: ev.result.output });
      } else if (ev.kind === "approval_request") {
        addApproval(ev.agent_id, ev.request);
      } else if (ev.kind === "done") {
        setChatGenerating(false);
        setActiveAgentId(null);
        if (ev.final_text) addChatMessage({ role: "assistant", content: ev.final_text });
        ipc.chatSaveHistory(useStore.getState().chatMessages).catch(() => {});
      } else if (ev.kind === "error") {
        setChatGenerating(false);
        setActiveAgentId(null);
        setChatError(ev.message);
      }
    }).then((fn) => unlisteners.push(fn));

    onConfigChanged((cfg) => {
      setConfig(cfg);
      applyTheme(cfg);
    }).then((fn) => unlisteners.push(fn));

    onServerLog((line) => {
      addServerLog(line);
    }).then((fn) => unlisteners.push(fn));

    return () => { unlisteners.forEach((fn) => fn()); };
  }, []);

  return (
    <TooltipProvider delayDuration={300}>
      <div className="flex h-full flex-col overflow-hidden">
        <Titlebar />
        <div className="flex flex-1 overflow-hidden">
          <Sidebar />
          <main className="flex flex-1 flex-col overflow-hidden">
            <TabContent tab={activeTab} />
          </main>
        </div>
      </div>
    </TooltipProvider>
  );
}

function TabContent({ tab }: { tab: string }) {
  switch (tab) {
    case "chat": return <ChatTab />;
    case "settings": return <SettingsTab />;
    case "overview": return <OverviewTab />;
    case "observability": return <ObservabilityTab />;
    case "notes": return <NotesTab />;
    case "todos": return <TodosTab />;
    case "calendar": return <CalendarTab />;
    case "planner": return <PlannerTab />;
    case "library": return <LibraryTab />;
    case "downloader": return <DownloaderTab />;
    case "instances": return <InstancesTab />;
    case "monitor": return <MonitorTab />;
    case "agents": return <AgentsTab />;
    case "mcp": return <McpTab />;
    case "compare": return <CompareTab />;
    case "deep-research": return <DeepResearchTab />;
    default: return <ChatTab />;
  }
}
