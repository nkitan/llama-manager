import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// ── Event channel names ──────────────────────────────────────────────────────
export const CHAT_EVENT = "chat://event";
export const AGENT_EVENT = "agent://event";
export const CONFIG_CHANGED_EVENT = "config://changed";
export const SERVER_LOG_EVENT = "server://log";

// ── Shared types (mirrors crates/shared/src/ipc.rs) ─────────────────────────
export interface ChatMessage {
  role: string;
  content: string;
}

export interface ChatRequest {
  stream_id: string;
  host: string;
  port: number;
  model: string;
  messages: ChatMessage[];
  temperature: number;
}

export interface ModelList {
  online: boolean;
  models: string[];
}

export type ChatEvent =
  | { kind: "token"; stream_id: string; delta: string }
  | { kind: "done"; stream_id: string }
  | { kind: "error"; stream_id: string; message: string };

export interface PlanStep {
  id: string;
  description: string;
  status: "pending" | "running" | "done" | "failed";
}

export interface ToolCall {
  call_id: string;
  tool: string;
  args: unknown;
  sensitive: boolean;
}

export interface ToolResult {
  call_id: string;
  ok: boolean;
  output: string;
}

export interface ApprovalRequest {
  call_id: string;
  tool: string;
  args: unknown;
  reason: string;
}

export interface AgentRequest {
  host: string;
  port: number;
  model: string;
  task: string;
}

export interface ApprovalDecision {
  agent_id: string;
  call_id: string;
  approved: boolean;
}

export type AgentEvent =
  | { kind: "started"; agent_id: string; parent: string | null; role: string; task: string }
  | { kind: "plan"; agent_id: string; steps: PlanStep[] }
  | { kind: "plan_update"; agent_id: string; step_id: string; status: string }
  | { kind: "step"; agent_id: string; index: number; thought: string }
  | { kind: "token"; agent_id: string; delta: string }
  | { kind: "tool_call"; agent_id: string; call: ToolCall }
  | { kind: "tool_result"; agent_id: string; result: ToolResult }
  | { kind: "approval_request"; agent_id: string; request: ApprovalRequest }
  | { kind: "sub_agent_spawned"; agent_id: string; child_id: string; role: string; task: string }
  | { kind: "log"; agent_id: string; message: string }
  | { kind: "done"; agent_id: string; final_text: string }
  | { kind: "error"; agent_id: string; message: string };

export interface TodoItem {
  id: string;
  text: string;
  done: boolean;
  priority: string;
  due_date: string | null;
  tags: string[];
}

export interface Note {
  name: string;
  content: string;
  tags: string[];
  pinned: boolean;
}

export interface NotesStore {
  notes: Note[];
}

export interface CalendarEvent {
  id: string;
  title: string;
  time: string;
  prompt: string;
  note: string | null;
  color: string | null;
  status: "pending" | "firing" | "completed" | "failed";
  result: string | null;
  tags: string[];
  recurrence: string | null;
}

export interface CalendarState {
  events: CalendarEvent[];
}

export interface KanbanTask {
  id: string;
  title: string;
  summary: string;
  assigned_agent: string;
  status: "backlog" | "todo" | "in_progress" | "done";
  priority: string;
  created_at: string;
}

export interface PlannerState {
  tasks: KanbanTask[];
}

// ── ServerConfig (partial — all UI-relevant fields) ──────────────────────────
export interface ServerConfig {
  exe_path: string;
  model_path: string;
  model_dir: string;
  model_alias: string;
  model_url: string;
  hf_repo: string;
  hf_file: string;
  chat_template: string;
  system_prompt: string;
  host: string;
  port: number;
  ctx_size: number;
  predict: number;
  gpu_layers: number;
  threads: number;
  threads_batch: number;
  threads_http: number;
  batch_size: number;
  ubatch_size: number;
  temperature: number;
  top_k: number;
  top_p: number;
  flash_attn: boolean;
  cont_batching: boolean;
  parallel: number;
  searxng_url: string;
  // UI theme
  dark_mode: boolean;
  theme_name: string;
  ui_transparency: number;
  ui_background_color: string;
  ui_text_color: string;
  ui_accent_color: string;
  ui_card_bg: string;
  ui_sidebar_bg: string;
  ui_border_color: string;
  ui_button_bg: string;
  ui_button_text: string;
  ui_card_text: string;
  ui_font_family: string;
  ui_blur: boolean;
  ui_blur_intensity: number;
  ui_radius_sm: string;
  ui_radius_md: string;
  ui_radius_lg: string;
  ui_radius_xl: string;
  ui_light_bg: string;
  ui_light_text: string;
  ui_light_accent: string;
  ui_light_card_bg: string;
  ui_light_sidebar_bg: string;
  ui_light_border: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [key: string]: any;
}

// ── Command wrappers ─────────────────────────────────────────────────────────
export const ipc = {
  getConfig: () => invoke<ServerConfig>("get_config"),
  updateConfig: (config: ServerConfig) => invoke<void>("update_config", { config }),

  chatListModels: (host: string, port: number) =>
    invoke<ModelList>("chat_list_models", { host, port }),
  chatSend: (req: ChatRequest) => invoke<void>("chat_send", { req }),
  chatSaveHistory: (history: ChatMessage[]) =>
    invoke<void>("chat_save_history", { history }),
  chatLoadHistory: () => invoke<ChatMessage[]>("chat_load_history"),

  agentStart: (req: AgentRequest) => invoke<string>("agent_start", { req }),
  agentApprove: (decision: ApprovalDecision) =>
    invoke<void>("agent_approve", { decision }),
  agentCancel: (agentId: string) => invoke<void>("agent_cancel", { agentId: agent_id }),
  agentStatus: () => invoke<string[]>("agent_status"),

  serverStart: () => invoke<void>("server_start"),
  serverStop: () => invoke<void>("server_stop"),
  serverStatus: () => invoke<boolean>("server_status"),
  pickPath: (directory: boolean) => invoke<string | null>("pick_path", { directory }),

  todosGet: () => invoke<TodoItem[]>("todos_get"),
  todosSet: (items: TodoItem[]) => invoke<void>("todos_set", { items }),

  notesGet: () => invoke<string>("notes_get"),
  notesSet: (content: string) => invoke<void>("notes_set", { content }),
  notesStoreGet: (scope: string) => invoke<NotesStore>("notes_store_get", { scope }),
  notesStoreSet: (scope: string, store: NotesStore) =>
    invoke<void>("notes_store_set", { scope, store }),

  calendarLoad: () => invoke<CalendarState>("calendar_load"),
  calendarSave: (events: CalendarEvent[]) =>
    invoke<void>("calendar_save", { events }),
  calendarAddEvent: (event: CalendarEvent) =>
    invoke<CalendarState>("calendar_add_event", { event }),
  calendarDeleteEvent: (id: string) =>
    invoke<CalendarState>("calendar_delete_event", { id }),

  plannerLoad: () => invoke<PlannerState>("planner_load"),
  plannerSave: (tasks: KanbanTask[]) => invoke<void>("planner_save", { tasks }),

  winMinimize: () => invoke<void>("win_minimize"),
  winToggleMaximize: () => invoke<void>("win_toggle_maximize"),
  winClose: () => invoke<void>("win_close"),
};

// Fix the agentCancel typo above
ipc.agentCancel = (agentId: string) => invoke<void>("agent_cancel", { agent_id: agentId });

// ── Typed event listeners ────────────────────────────────────────────────────
export const onChatEvent = (cb: (ev: ChatEvent) => void): Promise<UnlistenFn> =>
  listen<ChatEvent>(CHAT_EVENT, (e) => cb(e.payload));

export const onAgentEvent = (cb: (ev: AgentEvent) => void): Promise<UnlistenFn> =>
  listen<AgentEvent>(AGENT_EVENT, (e) => cb(e.payload));

export const onConfigChanged = (cb: (cfg: ServerConfig) => void): Promise<UnlistenFn> =>
  listen<ServerConfig>(CONFIG_CHANGED_EVENT, (e) => cb(e.payload));

export const onServerLog = (cb: (line: string) => void): Promise<UnlistenFn> =>
  listen<string>(SERVER_LOG_EVENT, (e) => cb(e.payload));
