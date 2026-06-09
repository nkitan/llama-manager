import { useEffect, useState, useMemo } from "react";
import { Play, Square, RefreshCw, FileText, Loader2 } from "lucide-react";
import { ipc, type ResearchStatus, type ResearchReportInfo } from "@/lib/ipc";
import { resolveTarget } from "@/lib/target";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";

const FALLBACK_MODELS = [
  "gemini-1.5-pro",
  "gemini-1.5-flash",
  "gpt-4o",
  "claude-3-5-sonnet",
  "llama3",
  "hermes-2-pro",
];

export function DeepResearchTab() {
  const [query, setQuery] = useState("");
  const [host, setHost] = useState("");
  const [port, setPort] = useState("");
  const [model, setModel] = useState("");
  const [availableModels, setAvailableModels] = useState<string[]>([]);
  const [customRevealed, setCustomRevealed] = useState(false);
  const [status, setStatus] = useState<ResearchStatus>({ is_running: false, log: "", report: "" });
  const [reports, setReports] = useState<ResearchReportInfo[]>([]);
  const [activeReport, setActiveReport] = useState("");
  const [reportContent, setReportContent] = useState("");
  const [statusText, setStatusText] = useState("");

  useEffect(() => {
    const { host: h, port: p, model: m } = resolveTarget("research");
    setHost(h);
    setPort(String(p));
    if (!model) setModel(m);

    (async () => {
      const all: string[] = [];
      try {
        const list = await ipc.chatListModels(h, p);
        for (const m of list.models) {
          if (m && !all.includes(m)) all.push(m);
        }
      } catch {}
      try {
        const scanned = await ipc.libraryGetIndex();
        for (const m of scanned) {
          if (m.clean_name && !all.includes(m.clean_name)) all.push(m.clean_name);
          if (m.filename && !all.includes(m.filename)) all.push(m.filename);
        }
      } catch {}
      for (const f of FALLBACK_MODELS) {
        if (!all.includes(f)) all.push(f);
      }
      setAvailableModels(all);
    })();
  }, []);

  useEffect(() => {
    const loadReports = async () => {
      try {
        const list = await ipc.deepResearchListReports();
        setReports(list);
      } catch {}
    };
    const check = async () => {
      try {
        const s = await ipc.deepResearchStatus();
        setStatus(s);
      } catch {}
    };
    loadReports();
    check();
    const t = setInterval(async () => {
      await check();
    }, 2000);
    return () => clearInterval(t);
  }, []);

  const showCustomInput = customRevealed || (model !== "" && !availableModels.includes(model));

  const selectValue = useMemo(() => {
    if (availableModels.includes(model)) return model;
    if (model === "" && !customRevealed) return "";
    return "__custom__";
  }, [model, availableModels, customRevealed]);

  const onSelectModel = (val: string) => {
    if (val === "__custom__") {
      setCustomRevealed(true);
      setModel("");
    } else {
      setCustomRevealed(false);
      setModel(val);
    }
  };

  const start = async () => {
    setStatusText("Launching deep research Python agent…");
    try {
      await ipc.deepResearchStart(query, host, Number(port) || 8080, model);
      setStatusText("Research running in background.");
      setStatus((s) => ({ ...s, is_running: true }));
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setStatusText(`Failed: ${msg}`);
    }
  };

  const cancel = async () => {
    try {
      await ipc.deepResearchCancel();
      setStatus((s) => ({ ...s, is_running: false }));
      setStatusText("Research campaign cancelled.");
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setStatusText(`Cancel failed: ${msg}`);
    }
  };

  const readReport = async (filename: string) => {
    setActiveReport(filename);
    try {
      const c = await ipc.deepResearchReadReport(filename);
      setReportContent(c);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setReportContent(`Error reading report: ${msg}`);
    }
  };

  return (
    <div className="flex flex-col h-full overflow-hidden p-3 gap-3">
      <div>
        <h2 className="text-sm font-semibold">Deep Research Labs</h2>
        <p className="text-[10px] text-muted-foreground">
          Conduct iterative research campaigns wrapped around deep_research.py.
        </p>
      </div>

      <div className="flex flex-wrap gap-3 flex-1 min-h-0">
        <div className="flex-1 min-w-[320px] flex flex-col gap-3 min-h-0">
          <div className="glass-card p-3 space-y-2.5 flex-shrink-0">
            <p className="text-[11px] font-semibold">Start Research Campaign</p>
            <div className="space-y-1">
              <span className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                Target Query / Research Topic
              </span>
              <Textarea
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="e.g. Research Llama 3 alignment updates and post-training methods."
                className="min-h-[80px] text-[12.5px]"
              />
            </div>
            <div className="space-y-1">
              <span className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                Model
              </span>
              <select
                value={selectValue}
                onChange={(e) => onSelectModel(e.target.value)}
                className="w-full h-8 text-[12px] bg-transparent border border-input rounded-md px-2 no-drag"
              >
                <option value="">-- Select Model --</option>
                {availableModels.map((m) => (
                  <option key={m} value={m}>{m}</option>
                ))}
                <option value="__custom__">Custom Model…</option>
              </select>
              {showCustomInput && (
                <Input
                  value={model}
                  onChange={(e) => setModel(e.target.value)}
                  placeholder="Enter custom model name…"
                  className="h-8 text-[12px]"
                />
              )}
            </div>
            <div className="flex gap-2">
              <Button
                onClick={start}
                disabled={status.is_running}
                size="sm"
                className="h-8 flex-1"
              >
                {status.is_running ? (
                  <><Loader2 className="h-3.5 w-3.5 animate-spin" /> Running…</>
                ) : (
                  <><Play className="h-3.5 w-3.5" /> Launch Research</>
                )}
              </Button>
              <Button
                onClick={cancel}
                disabled={!status.is_running}
                size="sm"
                variant="destructive"
                className="h-8"
              >
                <Square className="h-3.5 w-3.5" /> Cancel
              </Button>
            </div>
            {statusText && (
              <p className="text-[11px] font-medium text-muted-foreground">{statusText}</p>
            )}
          </div>

          <div className="glass-card p-3 space-y-2 flex flex-col flex-1 min-h-0">
            <p className="text-[11px] font-semibold flex-shrink-0">Live Timeline Log</p>
            <ScrollArea className="flex-1 min-h-0 bg-muted/30 rounded-md p-2">
              {status.log ? (
                <pre className="text-[11px] font-mono whitespace-pre-wrap break-words">
                  {status.log}
                </pre>
              ) : (
                <p className="text-[10.5px] text-muted-foreground italic">
                  Logs will appear here once deep research launches.
                </p>
              )}
            </ScrollArea>
          </div>
        </div>

        <div className="flex-[2] min-w-[480px] flex flex-col glass-card p-3 min-h-0">
          <div className="flex items-center justify-between border-b border-border/40 pb-2.5 mb-2.5 flex-shrink-0 gap-2 flex-wrap">
            <div>
              <p className="text-[14px] font-semibold">Research Reports & Insights</p>
              <p className="text-[11px] text-muted-foreground">
                Select a saved markdown report to view deep analysis
              </p>
            </div>
            <div className="flex items-center gap-2">
              <select
                value={activeReport}
                onChange={(e) => e.target.value && readReport(e.target.value)}
                className="h-8 text-[12px] bg-transparent border border-input rounded-md px-2 min-w-[180px] no-drag"
              >
                <option value="">Select a report…</option>
                {reports.map((r) => (
                  <option key={r.filename} value={r.filename}>{r.filename}</option>
                ))}
              </select>
              <Button
                size="sm" variant="outline"
                onClick={async () => {
                  try {
                    const list = await ipc.deepResearchListReports();
                    setReports(list);
                  } catch {}
                }}
                className="h-8"
              >
                <RefreshCw className="h-3 w-3" /> Refresh
              </Button>
            </div>
          </div>

          <ScrollArea className="flex-1 min-h-0 bg-muted/30 rounded-md p-3">
            {reportContent ? (
              <pre className="text-[13px] whitespace-pre-wrap break-words font-sans leading-relaxed">
                {reportContent}
              </pre>
            ) : (
              <div className="h-full min-h-[300px] flex flex-col items-center justify-center text-muted-foreground gap-2">
                <FileText className="h-7 w-7 opacity-50" />
                <p className="text-[12px] italic">
                  Select a report from the dropdown above to view its contents.
                </p>
              </div>
            )}
          </ScrollArea>
        </div>
      </div>
    </div>
  );
}
