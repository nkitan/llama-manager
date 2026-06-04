import { useRef, useEffect, useState, useCallback } from "react";
import { Send, Square, ChevronDown, BarChart2, Trash2, AlertCircle, Check, X } from "lucide-react";
import { useStore } from "@/store/app-store";
import { ipc } from "@/lib/ipc";
import { resolveTarget } from "@/lib/target";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { cn, estimateTokens } from "@/lib/utils";

export function ChatTab() {
  const {
    chatMessages, chatGenerating, chatError, chatDraft, chatShowContext,
    setChatDraft, setChatGenerating, setChatStreamId, setChatError,
    addChatMessage, updateLastAssistant, toggleChatContext,
    agentPlan, agentTrace, agentApprovals, removeApproval, addApproval,
    activeAgentId, setActiveAgentId, setAgentPlan, clearAgentTrace,
    config, serverOnline, serverModels,
  } = useStore();

  const bottomRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const [showTrace, setShowTrace] = useState(false);
  const [showPlan, setShowPlan] = useState(false);

  // Auto-scroll to bottom
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [chatMessages.length, chatGenerating]);

  const model = serverModels[0] ?? config?.model_alias ?? config?.model_path ?? "";
  const ctxSize = config?.ctx_size ?? 4096;
  const totalTokens = chatMessages.reduce((s, m) => s + estimateTokens(m.content), 0);
  const fillPct = Math.min(100, (totalTokens / ctxSize) * 100);

  const send = useCallback(async () => {
    if (!chatDraft.trim() || chatGenerating || !config) return;
    const userMsg = { role: "user", content: chatDraft.trim() };
    setChatDraft("");
    addChatMessage(userMsg);
    setChatGenerating(true);
    setChatError(null);

    const streamId = `chat-${Date.now()}`;
    setChatStreamId(streamId);

    const messages = [...useStore.getState().chatMessages];
    if (config.system_prompt) {
      messages.unshift({ role: "system", content: config.system_prompt });
    }

    const target = resolveTarget("planner");
    try {
      await ipc.chatSend({
        stream_id: streamId,
        host: target.host,
        port: target.port,
        model: target.model || model,
        messages,
        temperature: config.temperature ?? 0.7,
      });
    } catch (e) {
      setChatGenerating(false);
      setChatStreamId(null);
      setChatError(String(e));
    }
  }, [chatDraft, chatGenerating, config, model]);

  const cancel = useCallback(async () => {
    const agId = useStore.getState().activeAgentId;
    if (agId) {
      await ipc.agentCancel(agId).catch(() => {});
      setActiveAgentId(null);
    }
    setChatGenerating(false);
    setChatStreamId(null);
  }, []);

  const clearChat = useCallback(() => {
    useStore.getState().setChatMessages([]);
    setAgentPlan([]);
    clearAgentTrace();
    ipc.chatSaveHistory([]).catch(() => {});
  }, []);

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-border/40 flex-shrink-0" style={{ background: "hsl(var(--card) / 0.4)" }}>
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium">Chat</span>
          {serverOnline ? (
            <Badge variant="success" className="text-[10px] py-0">{model.split("/").pop()?.replace(/\.gguf$/i, "") || "online"}</Badge>
          ) : (
            <Badge variant="outline" className="text-[10px] py-0 text-muted-foreground">Offline</Badge>
          )}
          {chatGenerating && (
            <div className="h-1.5 w-1.5 rounded-full bg-primary animate-pulse" />
          )}
        </div>
        <div className="flex items-center gap-1">
          {agentTrace.length > 0 && (
            <Button variant="ghost" size="icon-sm" onClick={() => setShowTrace(!showTrace)} title="Agent trace">
              <span className="text-xs">🧠</span>
            </Button>
          )}
          {agentPlan.length > 0 && (
            <Button variant="ghost" size="icon-sm" onClick={() => setShowPlan(!showPlan)} title="Plan">
              <span className="text-xs">📋</span>
            </Button>
          )}
          <Button variant="ghost" size="icon-sm" onClick={toggleChatContext} title="Context overview">
            <BarChart2 className="h-3.5 w-3.5" />
          </Button>
          <Button variant="ghost" size="icon-sm" onClick={clearChat} title="Clear chat">
            <Trash2 className="h-3.5 w-3.5" />
          </Button>
        </div>
      </div>

      {/* Context overview panel */}
      {chatShowContext && (
        <div className="glass-card mx-3 mt-2 p-3 flex-shrink-0 animate-fade-in">
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs font-medium">Context Window</span>
            <span className="text-xs text-muted-foreground">~{totalTokens.toLocaleString()} / {ctxSize.toLocaleString()} tokens</span>
          </div>
          <Progress
            value={fillPct}
            className={cn("h-1.5", fillPct > 80 ? "text-destructive" : fillPct > 60 ? "text-amber-400" : "")}
          />
          <div className="mt-2 space-y-1 max-h-32 overflow-y-auto">
            {chatMessages.map((m, i) => (
              <div key={i} className="flex items-start gap-2 text-[10px]">
                <span className={cn("font-mono shrink-0", m.role === "user" ? "text-primary" : "text-muted-foreground")}>
                  {m.role === "user" ? "U" : "A"}
                </span>
                <span className="text-muted-foreground truncate flex-1">{m.content.slice(0, 60)}</span>
                <span className="text-muted-foreground/60 shrink-0">~{estimateTokens(m.content)}t</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Plan panel */}
      {showPlan && agentPlan.length > 0 && (
        <div className="glass-card mx-3 mt-2 p-3 flex-shrink-0 animate-fade-in">
          <p className="text-xs font-medium mb-2">Agent Plan</p>
          <div className="space-y-1">
            {agentPlan.map((step) => (
              <div key={step.id} className="flex items-start gap-2 text-xs">
                <PlanStatusIcon status={step.status} />
                <span className={cn(step.status === "done" ? "line-through text-muted-foreground" : "")}>{step.description}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Messages */}
      <ScrollArea className="flex-1 px-4">
        <div className="py-4 space-y-3">
          {chatMessages.length === 0 && (
            <div className="flex flex-col items-center justify-center h-full mt-16 text-center gap-3">
              <span className="text-4xl">🦙</span>
              <p className="text-sm text-muted-foreground">Start a conversation</p>
              {!serverOnline && (
                <p className="text-xs text-muted-foreground/60">Server is offline — start it from the sidebar</p>
              )}
            </div>
          )}
          {chatMessages.map((msg, i) => (
            <MessageBubble key={i} role={msg.role} content={msg.content} />
          ))}
          {chatGenerating && <TypingIndicator />}
          {chatError && (
            <div className="flex items-center gap-2 text-xs text-destructive glass-card p-2 rounded-md">
              <AlertCircle className="h-3.5 w-3.5 flex-shrink-0" />
              {chatError}
            </div>
          )}
          <div ref={bottomRef} />
        </div>
      </ScrollArea>

      {/* Approval modals */}
      {agentApprovals.map((a) => (
        <ApprovalModal
          key={a.request.call_id}
          agentId={a.agentId}
          request={a.request}
          onDecide={(approved) => {
            ipc.agentApprove({ agent_id: a.agentId, call_id: a.request.call_id, approved }).catch(() => {});
            removeApproval(a.request.call_id);
          }}
        />
      ))}

      {/* Trace drawer */}
      {showTrace && agentTrace.length > 0 && (
        <div className="glass-card mx-3 mb-2 p-2 max-h-40 overflow-y-auto animate-fade-in">
          <p className="text-[10px] font-semibold text-muted-foreground mb-1">Agent Trace</p>
          {agentTrace.map((line, i) => (
            <p key={i} className="text-[10px] font-mono text-muted-foreground">{line}</p>
          ))}
        </div>
      )}

      {/* Composer */}
      <div className="px-3 pb-3 pt-2 flex-shrink-0">
        <div className="flex gap-2 items-end glass-card p-2">
          <Textarea
            ref={textareaRef}
            value={chatDraft}
            onChange={(e) => {
              setChatDraft(e.target.value);
              e.target.style.height = "auto";
              e.target.style.height = Math.min(e.target.scrollHeight, 160) + "px";
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                send();
              }
            }}
            placeholder={serverOnline ? "Message… (Enter to send, Shift+Enter for newline)" : "Server offline"}
            disabled={!serverOnline && !chatGenerating}
            className="flex-1 border-0 bg-transparent shadow-none focus-visible:ring-0 text-sm min-h-[36px] py-1.5 resize-none"
            rows={1}
          />
          <Button
            size="icon-sm"
            variant={chatGenerating ? "destructive" : "default"}
            onClick={chatGenerating ? cancel : send}
            disabled={!chatGenerating && (!chatDraft.trim() || !serverOnline)}
            className="flex-shrink-0 mb-0.5"
          >
            {chatGenerating ? <Square className="h-3.5 w-3.5" /> : <Send className="h-3.5 w-3.5" />}
          </Button>
        </div>
        <p className="mt-1 text-[10px] text-center text-muted-foreground/40">
          {Math.round(fillPct)}% context used · {chatMessages.length} messages
        </p>
      </div>
    </div>
  );
}

function MessageBubble({ role, content }: { role: string; content: string }) {
  if (role === "system") return null;
  return (
    <div className={cn("flex", role === "user" ? "justify-end" : "justify-start")}>
      <div className={role === "user" ? "msg-user" : "msg-assistant"}>
        <pre className="whitespace-pre-wrap font-sans text-sm leading-relaxed">{content}</pre>
      </div>
    </div>
  );
}

function TypingIndicator() {
  return (
    <div className="flex justify-start">
      <div className="msg-assistant flex items-center gap-1 py-3">
        {[0, 1, 2].map((i) => (
          <div
            key={i}
            className="h-1.5 w-1.5 rounded-full bg-muted-foreground/60 animate-pulse"
            style={{ animationDelay: `${i * 0.15}s` }}
          />
        ))}
      </div>
    </div>
  );
}

function PlanStatusIcon({ status }: { status: string }) {
  if (status === "done") return <Check className="h-3 w-3 text-emerald-400 flex-shrink-0 mt-0.5" />;
  if (status === "running") return <div className="h-3 w-3 rounded-full border border-primary border-t-transparent animate-spin flex-shrink-0 mt-0.5" />;
  if (status === "failed") return <X className="h-3 w-3 text-destructive flex-shrink-0 mt-0.5" />;
  return <div className="h-3 w-3 rounded-full border border-muted-foreground/40 flex-shrink-0 mt-0.5" />;
}

function ApprovalModal({
  agentId, request, onDecide,
}: {
  agentId: string;
  request: { call_id: string; tool: string; args: unknown; reason: string };
  onDecide: (approved: boolean) => void;
}) {
  return (
    <Dialog open>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Tool Approval Required</DialogTitle>
        </DialogHeader>
        <div className="space-y-3">
          <div className="glass-card p-3 space-y-2">
            <div className="flex items-center gap-2">
              <Badge variant="warning">Sensitive</Badge>
              <span className="text-sm font-mono font-semibold">{request.tool}</span>
            </div>
            <p className="text-xs text-muted-foreground">{request.reason}</p>
            <pre className="text-[10px] font-mono bg-muted/30 rounded p-2 overflow-x-auto">
              {JSON.stringify(request.args, null, 2)}
            </pre>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onDecide(false)}>Deny</Button>
          <Button onClick={() => onDecide(true)}>Approve</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
