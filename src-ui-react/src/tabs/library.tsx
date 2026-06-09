import { useEffect, useMemo, useState } from "react";
import { Search, RefreshCw, Sparkles, Loader2, CheckCircle2, Download, XCircle, FolderInput } from "lucide-react";
import { ipc, type ScannedModel } from "@/lib/ipc";
import { useStore } from "@/store/app-store";
import { patchConfig } from "@/lib/target";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

export function LibraryTab() {
  const [index, setIndex] = useState<ScannedModel[]>([]);
  const [search, setSearch] = useState("");
  const [statusText, setStatusText] = useState("");
  const [scanning, setScanning] = useState(false);
  const [collapsedFamilies, setCollapsedFamilies] = useState<Set<string>>(new Set());
  const [collapsedVersions, setCollapsedVersions] = useState<Set<string>>(new Set());
  const { config, setActiveTab, serverOnline, addObsEvent } = useStore();

  const loadData = async () => {
    try {
      const idx = await ipc.libraryGetIndex();
      setIndex(idx);
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const scan = async () => {
    setScanning(true);
    setStatusText("Scanning directories…");
    try {
      const idx = await ipc.libraryScan();
      setIndex(idx);
      setStatusText("Scan complete.");
    } catch (e) {
      setStatusText(`Scan failed: ${e}`);
    } finally {
      setScanning(false);
    }
  };

  const enrichSingle = async (path: string) => {
    setStatusText(`Enriching model: ${path}…`);
    try {
      const idx = await ipc.libraryEnrichSingle(path);
      setIndex(idx);
      setStatusText("Enrichment complete.");
    } catch (e) {
      setStatusText(`Enrichment failed: ${e}`);
    }
  };

  const enrichAll = async () => {
    setStatusText("Enriching all pending models in background…");
    try {
      await ipc.libraryEnrichAll();
    } catch (e) {
      setStatusText(`Failed to start enrich all: ${e}`);
    }
  };

  const useModel = async (m: ScannedModel) => {
    addObsEvent({ ts: Date.now(), kind: "library:use_model", id: m.filename, content: m.path });
    setStatusText("Active model updated!");
    await patchConfig({ model_path: m.path, model_alias: m.clean_name || m.filename });
    setActiveTab("chat");
    if (serverOnline) {
      try {
        await ipc.serverStop();
        await ipc.serverStart();
      } catch {
        /* server may not be running */
      }
    }
  };

  const filtered = useMemo(() => {
    const q = search.toLowerCase();
    return index.filter(
      (m) =>
        m.filename.toLowerCase().includes(q) ||
        m.clean_name.toLowerCase().includes(q) ||
        m.family.toLowerCase().includes(q) ||
        m.use_case.toLowerCase().includes(q) ||
        m.tags.some((t) => t.toLowerCase().includes(q))
    );
  }, [index, search]);

  const families = useMemo(() => {
    const set = new Set<string>();
    for (const m of filtered) {
      set.add(m.family || "Other");
    }
    return Array.from(set).sort((a, b) => a.toLowerCase().localeCompare(b.toLowerCase()));
  }, [filtered]);

  const toggleFamily = (fam: string) => {
    setCollapsedFamilies((s) => {
      const next = new Set(s);
      next.has(fam) ? next.delete(fam) : next.add(fam);
      return next;
    });
  };
  const toggleVersion = (key: string) => {
    setCollapsedVersions((s) => {
      const next = new Set(s);
      next.has(key) ? next.delete(key) : next.add(key);
      return next;
    });
  };

  const isActive = (m: ScannedModel) => {
    if (!config) return false;
    return config.model_path === m.path || config.model_alias === m.filename;
  };

  return (
    <div className="flex flex-col h-full overflow-hidden p-3 gap-3">
      <div className="flex items-center justify-between flex-shrink-0">
        <div>
          <h2 className="text-sm font-semibold">Model Index</h2>
          <p className="text-[10px] text-muted-foreground">
            Discover and enrich local model files and HuggingFace metadata.
          </p>
        </div>
        <div className="flex gap-1">
          <Button size="sm" variant="default" onClick={scan} disabled={scanning} className="h-7 text-xs">
            {scanning ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <Search className="h-3.5 w-3.5" />}
            Scan Directories
          </Button>
          <Button size="sm" variant="outline" onClick={enrichAll} className="h-7 text-xs">
            <Sparkles className="h-3.5 w-3.5" /> Enrich All
          </Button>
          <Button size="icon-sm" variant="ghost" onClick={loadData} title="Refresh">
            <RefreshCw className="h-3.5 w-3.5" />
          </Button>
        </div>
      </div>

      {statusText && (
        <div className="text-[11px] font-medium text-muted-foreground flex-shrink-0">
          {statusText}
        </div>
      )}

      <div className="relative flex-shrink-0">
        <Search className="absolute left-2.5 top-2.5 h-3.5 w-3.5 text-muted-foreground" />
        <Input
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search by filename, family, tag, or use-case…"
          className="pl-8 h-8 text-xs"
        />
      </div>

      <ScrollArea className="flex-1 glass-card rounded-lg p-2">
        {families.length === 0 ? (
          <div className="text-center py-12 text-xs text-muted-foreground">
            <FolderInput className="h-8 w-8 mx-auto mb-2 opacity-40" />
            No models found. Try scanning your model directories.
          </div>
        ) : (
          <div className="space-y-2">
            {families.map((family) => {
              const familyModels = filtered.filter(
                (m) => (m.family || "Other") === family
              );
              const isCollapsed = collapsedFamilies.has(family);
              const versions = Array.from(
                new Set(familyModels.map((m) => m.version || "Unknown"))
              ).sort((a, b) => a.toLowerCase().localeCompare(b.toLowerCase()));

              return (
                <div key={family} className="space-y-1">
                  <button
                    onClick={() => toggleFamily(family)}
                    className="flex w-full items-center justify-between rounded-md bg-accent/30 hover:bg-accent/50 px-2.5 py-1.5 transition-colors no-drag"
                  >
                    <span className="text-[11px] font-semibold flex items-center gap-1.5">
                      <span className="text-primary text-[10px]">
                        {isCollapsed ? "▶" : "▼"}
                      </span>
                      📁 {family} Family
                    </span>
                    <Badge variant="secondary" className="text-[9px]">
                      {familyModels.length} files
                    </Badge>
                  </button>

                  {!isCollapsed && (
                    <div className="space-y-1 ml-3">
                      {versions.map((version) => {
                        const verKey = `${family}:${version}`;
                        const verCollapsed = collapsedVersions.has(verKey);
                        const verModels = familyModels.filter(
                          (m) => (m.version || "Unknown") === version
                        );
                        return (
                          <div key={verKey} className="space-y-1">
                            <button
                              onClick={() => toggleVersion(verKey)}
                              className="flex w-full items-center justify-between rounded px-2 py-1 text-[10.5px] text-muted-foreground hover:bg-accent/30 transition-colors no-drag"
                            >
                              <span className="flex items-center gap-1.5">
                                <span className="text-[9px]">
                                  {verCollapsed ? "▶" : "▼"}
                                </span>
                                ↳ 🗄️ {version}
                              </span>
                              <span className="text-[10px]">{verModels.length} files</span>
                            </button>

                            {!verCollapsed && (
                              <div className="rounded-md border border-border/40 overflow-hidden">
                                <table className="w-full text-[10.5px]">
                                  <thead className="bg-muted/30">
                                    <tr className="border-b border-border/40">
                                      <th className="text-left px-2 py-1.5 font-medium text-muted-foreground">
                                        Model
                                      </th>
                                      <th className="text-left px-2 py-1.5 font-medium text-muted-foreground w-24">
                                        Use-case
                                      </th>
                                      <th className="text-left px-2 py-1.5 font-medium text-muted-foreground w-16">
                                        Size
                                      </th>
                                      <th className="text-left px-2 py-1.5 font-medium text-muted-foreground w-16">
                                        Links
                                      </th>
                                      <th className="text-right px-2 py-1.5 font-medium text-muted-foreground w-32">
                                        Action
                                      </th>
                                    </tr>
                                  </thead>
                                  <tbody>
                                    {verModels.map((m) => (
                                      <ModelRow
                                        key={m.path}
                                        model={m}
                                        active={isActive(m)}
                                        onEnrich={() => enrichSingle(m.path)}
                                        onUse={() => useModel(m)}
                                      />
                                    ))}
                                  </tbody>
                                </table>
                              </div>
                            )}
                          </div>
                        );
                      })}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </ScrollArea>
    </div>
  );
}

function ModelRow({
  model: m,
  active,
  onEnrich,
  onUse,
}: {
  model: ScannedModel;
  active: boolean;
  onEnrich: () => void;
  onUse: () => void;
}) {
  const sizeGB = m.size_bytes / (1024 * 1024 * 1024);
  return (
    <tr
      className={cn(
        "border-b border-border/30 hover:bg-accent/20 transition-colors",
        active && "bg-primary/10"
      )}
    >
      <td className="px-2 py-1.5 max-w-[280px]">
        <div className="font-semibold flex items-center gap-1.5 flex-wrap text-[11px]">
          <span className="break-all">
            {m.clean_name && m.clean_name !== "Unknown" ? m.clean_name : m.filename}
          </span>
          <span className="text-[8px] bg-muted text-primary px-1 rounded font-bold uppercase">
            {m.version}
          </span>
          {active && (
            <Badge variant="success" className="text-[8px] py-0">
              Active
            </Badge>
          )}
          <StatusPill status={m.status} />
        </div>
        <div className="text-[9px] text-muted-foreground font-mono mt-0.5 break-all">
          {m.filename}
        </div>
        {m.tags.length > 0 && (
          <div className="flex flex-wrap gap-1 mt-0.5">
            {m.tags.map((t) => (
              <span
                key={t}
                className="text-[8px] bg-muted text-muted-foreground px-1 py-0.5 rounded"
              >
                {t}
              </span>
            ))}
          </div>
        )}
      </td>
      <td className="px-2 py-1.5">
        {m.use_case && (
          <UseCaseBadge useCase={m.use_case} />
        )}
      </td>
      <td className="px-2 py-1.5 text-muted-foreground whitespace-nowrap">
        {sizeGB.toFixed(2)} GB
      </td>
      <td className="px-2 py-1.5">
        <div className="flex gap-1.5 text-[10px]">
          {m.hf_link && (
            <a
              href={m.hf_link}
              target="_blank"
              rel="noreferrer"
              className="text-primary hover:underline"
            >
              HF
            </a>
          )}
          {m.github_link && (
            <a
              href={m.github_link}
              target="_blank"
              rel="noreferrer"
              className="text-emerald-400 hover:underline"
            >
              Git
            </a>
          )}
        </div>
      </td>
      <td className="px-2 py-1.5">
        <div className="flex gap-1 justify-end">
          {m.status !== "enriching" && m.status !== "enriched" && (
            <Button
              size="sm"
              variant="outline"
              onClick={onEnrich}
              className="h-5 text-[9px] px-1.5"
            >
              Enrich
            </Button>
          )}
          <Button
            size="sm"
            variant="default"
            onClick={onUse}
            className="h-5 text-[9px] px-1.5"
          >
            <Download className="h-2.5 w-2.5" /> Use
          </Button>
        </div>
      </td>
    </tr>
  );
}

function StatusPill({ status }: { status: string }) {
  if (status === "enriching")
    return (
      <span className="text-[8px] bg-amber-500/20 text-amber-400 px-1 rounded font-bold uppercase inline-flex items-center gap-0.5">
        <Loader2 className="h-2 w-2 animate-spin" /> Enriching
      </span>
    );
  if (status === "enriched")
    return (
      <span className="text-[8px] bg-emerald-500/20 text-emerald-400 px-1 rounded font-bold uppercase inline-flex items-center gap-0.5">
        <CheckCircle2 className="h-2 w-2" /> Enriched
      </span>
    );
  if (status === "failed")
    return (
      <span className="text-[8px] bg-destructive/20 text-destructive px-1 rounded font-bold uppercase inline-flex items-center gap-0.5">
        <XCircle className="h-2 w-2" /> Failed
      </span>
    );
  return null;
}

function UseCaseBadge({ useCase }: { useCase: string }) {
  const map: Record<string, string> = {
    "Text Generation": "bg-primary/15 text-primary",
    Coding: "bg-destructive/15 text-destructive",
    "Text Embedding": "bg-blue-500/15 text-blue-400",
  };
  const cls = map[useCase] ?? "bg-muted text-muted-foreground";
  return (
    <span className={cn("text-[9px] font-semibold px-1.5 py-0.5 rounded", cls)}>
      {useCase}
    </span>
  );
}
