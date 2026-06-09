import { useEffect, useRef, useState } from "react";
import { Play, Square, Terminal, AlertCircle } from "lucide-react";
import { ipc, type DownloadStatus } from "@/lib/ipc";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
  DrawerTrigger,
} from "@/components/ui/drawer";
import { cn } from "@/lib/utils";

interface TargetItem {
  name: string;
  line: string;
}

function parseInputTargets(input: string): TargetItem[] {
  const targets: TargetItem[] = [];
  const lines = input.split("\n");
  for (const raw of lines) {
    const trimmed = raw.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;
    if (trimmed.endsWith(":")) continue;
    if (trimmed.startsWith("-")) continue;

    let displayName: string;
    if (trimmed.startsWith("http")) {
      displayName = trimmed.split("/").pop() ?? trimmed;
    } else if (trimmed.endsWith(".git") || trimmed.startsWith("https://github")) {
      const repo = trimmed.split("/").pop() ?? trimmed;
      displayName = `git: ${repo.replace(/\.git$/, "")}`;
    } else {
      const repoName = trimmed.split("/").pop() ?? "";
      displayName = `HF: ${repoName}`;
    }
    targets.push({ name: displayName, line: trimmed });
  }
  return targets;
}

function extractDlPercentage(line: string): number | null {
  const pctPos = line.indexOf("%");
  if (pctPos === -1) return null;
  const before = line.slice(0, pctPos);
  let numStart = 0;
  for (let i = before.length - 1; i >= 0; i--) {
    const c = before[i];
    if (!(c >= "0" && c <= "9") && c !== ".") {
      numStart = i + 1;
      break;
    }
  }
  const num = before.slice(numStart);
  const v = parseFloat(num);
  return Number.isFinite(v) ? v : null;
}

function extractDlSpeed(line: string): string | null {
  const lower = line.toLowerCase();
  const dlIdx = lower.indexOf("dl:");
  if (dlIdx !== -1) {
    const sub = line.slice(dlIdx + 3).trim();
    const end = sub.indexOf("]");
    const sp = end === -1 ? sub : sub.slice(0, end);
    if (sp.trim()) return sp.trim();
  }
  const tokens = line.split(/\s+/);
  for (let i = tokens.length - 1; i >= 0; i--) {
    const t = tokens[i].replace(/[,\])]/g, "");
    if (t.endsWith("/s") || t.endsWith("b/s") || t.endsWith("B/s")) {
      return t;
    }
  }
  return null;
}

function extractDlEta(line: string): string | null {
  const lower = line.toLowerCase();
  const idx = lower.indexOf("eta");
  if (idx !== -1) {
    const after = line
      .slice(idx + 3)
      .replace(/^[:\s]+/, "")
      .trim();
    if (after) {
      return after.split(/\s+/)[0] ?? "Unknown";
    }
  }
  return null;
}

function parseTmuxProgress(
  log: string,
  targets: TargetItem[]
): { activeIdx: number | null; percent: number; speed: string; eta: string } {
  const lines = log.split("\n");
  let activeIdx: number | null = null;
  let percent = 0;
  let speed = "Pending…";
  let eta = "Unknown";

  for (let i = lines.length - 1; i >= 0; i--) {
    const line = lines[i];
    const lower = line.toLowerCase();
    for (let j = targets.length - 1; j >= 0; j--) {
      const target = targets[j];
      const kw = target.line.toLowerCase();
      const nameKw = target.name.toLowerCase();
      const fileKw = (target.line.split("/").pop() ?? "").toLowerCase();
      if (
        (kw && lower.includes(kw)) ||
        (nameKw && lower.includes(nameKw)) ||
        (fileKw && lower.includes(fileKw))
      ) {
        activeIdx = j;
        break;
      }
    }
    if (activeIdx !== null) break;
  }

  for (let i = lines.length - 1; i >= 0; i--) {
    const line = lines[i];
    const p = extractDlPercentage(line);
    if (p !== null) {
      percent = p;
      const s = extractDlSpeed(line);
      if (s) speed = s;
      const e = extractDlEta(line);
      if (e) eta = e;
      break;
    }
    if (line.includes("Receiving objects:") || line.includes("Resolving deltas:")) {
      const p = extractDlPercentage(line);
      if (p !== null) percent = p;
      const s = extractDlSpeed(line);
      if (s) speed = s;
      eta = "Cloning…";
      break;
    }
  }

  return { activeIdx, percent, speed, eta };
}

export function DownloaderTab() {
  const [targetsText, setTargetsText] = useState("");
  const [isDownloading, setIsDownloading] = useState(false);
  const [logData, setLogData] = useState("");
  const [errorMsg, setErrorMsg] = useState("");
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const loadTargets = async () => {
    try {
      const t = await ipc.downloadGetTargets();
      setTargetsText(t);
    } catch (e) {
      console.error(e);
    }
  };

  const checkStatus = async () => {
    try {
      const status: DownloadStatus = await ipc.downloadStatus();
      setIsDownloading(status.is_downloading);
      if (status.log) setLogData(status.log);
    } catch {
      /* ignore */
    }
  };

  useEffect(() => {
    loadTargets();
    checkStatus();
    pollRef.current = setInterval(checkStatus, 1500);
    return () => {
      if (pollRef.current) clearInterval(pollRef.current);
    };
  }, []);

  const start = async () => {
    setErrorMsg("");
    try {
      await ipc.downloadSaveTargets(targetsText);
      await ipc.downloadStart(targetsText);
      setIsDownloading(true);
    } catch (e) {
      setErrorMsg(`Failed to start: ${e}`);
    }
  };

  const cancel = async () => {
    try {
      await ipc.downloadCancel();
      setIsDownloading(false);
    } catch (e) {
      setErrorMsg(`Failed to cancel: ${e}`);
    }
  };

  const targets = parseInputTargets(targetsText);
  const { activeIdx, percent, speed, eta } = parseTmuxProgress(logData, targets);

  return (
    <div className="flex flex-col h-full overflow-hidden p-3 gap-3">
      <div className="flex items-center justify-between flex-shrink-0">
        <div>
          <h2 className="text-sm font-semibold">Download Manager</h2>
          <p className="text-[10px] text-muted-foreground">
            Model & repo downloader via tmux session <code className="bg-muted/40 px-1 rounded">hf_downloads</code>
          </p>
        </div>
      </div>

      {errorMsg && (
        <div className="flex items-start gap-2 text-[11px] text-destructive glass-card p-2 rounded-md flex-shrink-0">
          <AlertCircle className="h-3.5 w-3.5 flex-shrink-0 mt-0.5" />
          {errorMsg}
        </div>
      )}

      <div className="flex flex-1 min-h-0 gap-3 flex-wrap">
        {/* Left: target editor */}
        <div className="flex-1 min-w-[320px] flex flex-col gap-3">
          <div className="glass-card p-3 flex flex-col flex-1 min-h-0">
            <p className="text-[11px] font-semibold mb-1">Download Targets</p>
            <p className="text-[10px] text-muted-foreground mb-2">
              Format: <code className="bg-muted/40 px-1 rounded">category:</code> headers, <code className="bg-muted/40 px-1 rounded">-subfolder</code> lines, then one model/URL per line.
            </p>
            <textarea
              value={targetsText}
              onChange={(e) => setTargetsText(e.target.value)}
              disabled={isDownloading}
              placeholder={`# Example\ntext:\n- stable\nfacebook/opt-125m\n\nmultimodal:\nhttps://example.com/model.gguf`}
              className="flex-1 min-h-[260px] w-full resize-vertical rounded-md border border-input bg-transparent px-2 py-1.5 text-[11px] font-mono focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:opacity-50 no-drag"
              spellCheck={false}
            />
            <div className="flex gap-2 mt-2 flex-shrink-0">
              <Button
                size="sm"
                variant="default"
                onClick={start}
                disabled={isDownloading}
                className="flex-1 h-7 text-xs"
              >
                {isDownloading ? (
                  <>
                    <Square className="h-3.5 w-3.5" /> Downloading…
                  </>
                ) : (
                  <>
                    <Play className="h-3.5 w-3.5" /> Generate dw.sh & Start
                  </>
                )}
              </Button>
              <Button
                size="sm"
                variant="destructive"
                onClick={cancel}
                disabled={!isDownloading}
                className="h-7 text-xs"
              >
                Cancel
              </Button>
            </div>
            <p className="text-[10px] text-muted-foreground/70 mt-1.5">
              Tip: attach with{" "}
              <code className="bg-muted/40 px-1 rounded">tmux attach -t hf_downloads</code>
            </p>
          </div>
        </div>

        {/* Right: live monitor */}
        <div className="flex-1 min-w-[320px] flex flex-col gap-3">
          <div className="glass-card p-3 flex flex-col flex-1 min-h-0">
            <div className="flex items-center gap-2 mb-3">
              <div
                className={cn(
                  "h-2 w-2 rounded-full",
                  isDownloading ? "bg-emerald-400" : "bg-muted-foreground/40"
                )}
              />
              <p className="text-[11px] font-semibold">
                {isDownloading
                  ? "Downloading in 'hf_downloads' tmux session"
                  : "Idle / Completed"}
              </p>
            </div>

            <ScrollArea className="flex-1 min-h-0">
              <div className="space-y-2 pr-1">
                {targets.length === 0 ? (
                  <p className="text-center text-[11px] text-muted-foreground italic py-6">
                    No targets defined. Edit the file on the left.
                  </p>
                ) : (
                  targets.map((target, idx) => {
                    const isActive = isDownloading && activeIdx === idx;
                    const isDone = isDownloading
                      ? activeIdx !== null && idx < activeIdx
                      : !!logData;
                    let barWidth: string;
                    let barColor: string;
                    let label: string;
                    if (isActive) {
                      barWidth = `${Math.round(percent)}%`;
                      barColor = "hsl(var(--primary))";
                      label = `${Math.round(percent)}%`;
                    } else if (isDone) {
                      barWidth = "100%";
                      barColor = "#10b981";
                      label = "✅ Done";
                    } else {
                      barWidth = "0%";
                      barColor = "hsl(var(--muted-foreground) / 0.3)";
                      label = "⏳ Pending";
                    }
                    return (
                      <div
                        key={`${target.line}-${idx}`}
                        className="rounded-md border border-border/40 bg-muted/20 p-2 space-y-1.5"
                      >
                        <div className="flex items-center justify-between gap-2">
                          <span className="font-semibold text-[11px] break-all flex-1">
                            {target.name}
                          </span>
                          <span
                            className={cn(
                              "text-[10px] font-semibold whitespace-nowrap",
                              isActive
                                ? "text-primary"
                                : isDone
                                ? "text-emerald-400"
                                : "text-muted-foreground"
                            )}
                          >
                            {label}
                          </span>
                        </div>
                        <div className="w-full h-1 bg-muted rounded-full overflow-hidden">
                          <div
                            className="h-full rounded-full transition-all duration-300"
                            style={{ width: barWidth, background: barColor }}
                          />
                        </div>
                        {isActive && (
                          <div className="flex justify-between text-[10px] text-muted-foreground">
                            <span>Speed: {speed}</span>
                            <span>ETA: {eta}</span>
                          </div>
                        )}
                      </div>
                    );
                  })
                )}
              </div>
            </ScrollArea>

            {(isDownloading || logData) && (
              <div className="mt-3 flex-shrink-0">
                <Drawer>
                  <DrawerTrigger asChild>
                    <Button
                      size="sm"
                      variant="outline"
                      className="w-full h-7 text-[10px]"
                    >
                      <Terminal className="h-3 w-3" />
                      Open Raw Terminal Output
                    </Button>
                  </DrawerTrigger>
                  <DrawerContent>
                    <DrawerHeader>
                      <DrawerTitle>hf_downloads — raw log</DrawerTitle>
                      <DrawerDescription>
                        Live tmux capture from <code className="bg-muted/40 px-1 rounded">hf_downloads</code>
                      </DrawerDescription>
                    </DrawerHeader>
                    <div className="flex-1 min-h-0 px-3 pb-4">
                      <div className="h-[60vh] rounded-md border border-border/40 bg-black/40 p-2 overflow-auto">
                        <pre className="text-[10px] font-mono text-emerald-300/90 whitespace-pre-wrap break-all">
                          {logData || "(no output yet)"}
                        </pre>
                      </div>
                    </div>
                    <div className="px-3 pb-4 flex gap-2">
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => {
                          navigator.clipboard.writeText(logData).catch(() => {});
                        }}
                        className="flex-1 h-7 text-[10px]"
                      >
                        Copy to clipboard
                      </Button>
                    </div>
                  </DrawerContent>
                </Drawer>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
