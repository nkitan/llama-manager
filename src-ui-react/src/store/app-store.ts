import { create } from "zustand";
import type {
  ServerConfig,
  ChatMessage,
  PlanStep,
  ApprovalRequest,
} from "@/lib/ipc";

export interface ObsEvent {
  ts: number;
  kind: string;
  id: string;
  content: string;
}

const OBS_CAP = 4000;

interface AppStore {
  // ── Config ────────────────────────────────────────────────────────────────
  config: ServerConfig | null;
  setConfig: (cfg: ServerConfig) => void;

  // ── Navigation ────────────────────────────────────────────────────────────
  activeTab: string;
  setActiveTab: (tab: string) => void;

  // ── Chat (persists across tab switches) ──────────────────────────────────
  chatMessages: ChatMessage[];
  setChatMessages: (msgs: ChatMessage[]) => void;
  addChatMessage: (msg: ChatMessage) => void;
  updateLastAssistant: (delta: string) => void;
  chatGenerating: boolean;
  setChatGenerating: (v: boolean) => void;
  chatStreamId: string | null;
  setChatStreamId: (id: string | null) => void;
  chatError: string | null;
  setChatError: (e: string | null) => void;
  chatDraft: string;
  setChatDraft: (s: string) => void;
  chatShowContext: boolean;
  toggleChatContext: () => void;

  // ── Agent ─────────────────────────────────────────────────────────────────
  activeAgentId: string | null;
  setActiveAgentId: (id: string | null) => void;
  agentPlan: PlanStep[];
  setAgentPlan: (steps: PlanStep[]) => void;
  updatePlanStep: (stepId: string, status: PlanStep["status"]) => void;
  agentTrace: string[];
  addAgentTrace: (msg: string) => void;
  clearAgentTrace: () => void;
  agentApprovals: Array<{ agentId: string; request: ApprovalRequest }>;
  addApproval: (agentId: string, req: ApprovalRequest) => void;
  removeApproval: (callId: string) => void;

  // ── Observability ─────────────────────────────────────────────────────────
  obsEvents: ObsEvent[];
  addObsEvent: (ev: ObsEvent) => void;
  clearObsEvents: () => void;

  // ── Server ────────────────────────────────────────────────────────────────
  serverOnline: boolean;
  serverModels: string[];
  setServerStatus: (online: boolean, models: string[]) => void;
  serverLogs: string[];
  addServerLog: (line: string) => void;
  clearServerLogs: () => void;
}

export const useStore = create<AppStore>()((set) => ({
  // Config
  config: null,
  setConfig: (cfg) => set({ config: cfg }),

  // Nav
  activeTab: "chat",
  setActiveTab: (tab) => set({ activeTab: tab }),

  // Chat
  chatMessages: [],
  setChatMessages: (msgs) => set({ chatMessages: msgs }),
  addChatMessage: (msg) =>
    set((s) => ({ chatMessages: [...s.chatMessages, msg] })),
  updateLastAssistant: (delta) =>
    set((s) => {
      const msgs = [...s.chatMessages];
      const last = msgs[msgs.length - 1];
      if (last?.role === "assistant") {
        msgs[msgs.length - 1] = { ...last, content: last.content + delta };
      } else {
        msgs.push({ role: "assistant", content: delta });
      }
      return { chatMessages: msgs };
    }),
  chatGenerating: false,
  setChatGenerating: (v) => set({ chatGenerating: v }),
  chatStreamId: null,
  setChatStreamId: (id) => set({ chatStreamId: id }),
  chatError: null,
  setChatError: (e) => set({ chatError: e }),
  chatDraft: "",
  setChatDraft: (s) => set({ chatDraft: s }),
  chatShowContext: false,
  toggleChatContext: () => set((s) => ({ chatShowContext: !s.chatShowContext })),

  // Agent
  activeAgentId: null,
  setActiveAgentId: (id) => set({ activeAgentId: id }),
  agentPlan: [],
  setAgentPlan: (steps) => set({ agentPlan: steps }),
  updatePlanStep: (stepId, status) =>
    set((s) => ({
      agentPlan: s.agentPlan.map((p) =>
        p.id === stepId ? { ...p, status } : p
      ),
    })),
  agentTrace: [],
  addAgentTrace: (msg) =>
    set((s) => ({ agentTrace: [...s.agentTrace.slice(-199), msg] })),
  clearAgentTrace: () => set({ agentTrace: [] }),
  agentApprovals: [],
  addApproval: (agentId, req) =>
    set((s) => ({ agentApprovals: [...s.agentApprovals, { agentId, request: req }] })),
  removeApproval: (callId) =>
    set((s) => ({
      agentApprovals: s.agentApprovals.filter((a) => a.request.call_id !== callId),
    })),

  // Obs
  obsEvents: [],
  addObsEvent: (ev) =>
    set((s) => {
      const next = [...s.obsEvents, ev];
      return { obsEvents: next.length > OBS_CAP ? next.slice(-OBS_CAP) : next };
    }),
  clearObsEvents: () => set({ obsEvents: [] }),

  // Server
  serverOnline: false,
  serverModels: [],
  setServerStatus: (online, models) => set({ serverOnline: online, serverModels: models }),
  serverLogs: [],
  addServerLog: (line) =>
    set((s) => ({ serverLogs: [...s.serverLogs.slice(-999), line] })),
  clearServerLogs: () => set({ serverLogs: [] }),
}));
