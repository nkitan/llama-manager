import { useEffect, useState } from "react";
import { Search, Server, RotateCcw, Link2 } from "lucide-react";
import { ipc, type LlamaInstance, type ModelOverride } from "@/lib/ipc";
import { useStore } from "@/store/app-store";
import { patchConfig, patchOverride, type ToolScope } from "@/lib/target";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

const OVERRIDE_SCOPES: { id: ToolScope; label: string }[] = [
  { id: "planner", label: "Task Planner & Subagents" },
  { id: "calendar", label: "Calendar Scheduled Actions" },
  { id: "memory", label: "Quick Notes / Memory Copilot" },
  { id: "research", label: "Deep Research Agent" },
  { id: "compare", label: "Compare Tab / llama-benchy" },
];

export function InstancesTab() {
  const [detected, setDetected] = useState<LlamaInstance[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const {
    config,
    routedInstance,
    routedModel,
    setRoutedInstance,
    setRoutedModel,
  } = useStore();

  const scan = async () => {
    setIsScanning(true);
    try {
      const list = await ipc.instancesDetect();
      setDetected(list);
    } catch (e) {
      console.error(e);
    } finally {
      setIsScanning(false);
    }
  };

  useEffect(() => {
    scan();
  }, []);

  const setRoute = (host: string, port: number, defaultModel: string | null) => {
    setRoutedInstance({ host, port });
    setRoutedModel(defaultModel);
  };

  const resetRoute = () => {
    setRoutedInstance(null);
    setRoutedModel(null);
  };

  return (
    <div className="flex flex-col h-full overflow-hidden p-3 gap-3">
      <div>
        <h2 className="text-sm font-semibold">Instances</h2>
        <p className="text-[10px] text-muted-foreground">
          Scan running model server instances and route front-end requests.
        </p>
      </div>

      <div className="flex flex-1 min-h-0 gap-3 flex-wrap overflow-y-auto">
        <div className="flex-1 min-w-[320px] flex flex-col gap-3">
          <div className="glass-card p-3">
            <div className="flex items-start justify-between gap-2">
              <div className="min-w-0">
                <p className="text-[12px] font-semibold truncate">
                  {routedInstance
                    ? `Routed: http://${routedInstance.host}:${routedInstance.port}`
                    : "Routed: Local launch-config server"}
                </p>
                <p className="text-[11px] text-primary font-medium mt-0.5 truncate">
                  Active Model: {routedModel || "Default local model"}
                </p>
                <p className="text-[10px] text-muted-foreground mt-0.5">
                  Determines where Chat and agent modules send requests.
                </p>
              </div>
              {routedInstance && (
                <Button
                  size="sm"
                  variant="outline"
                  onClick={resetRoute}
                  className="h-7 text-[11px] flex-shrink-0"
                >
                  <RotateCcw className="h-3 w-3" /> Reset
                </Button>
              )}
            </div>
          </div>

          <div className="glass-card p-3 flex flex-col flex-1 min-h-0">
            <div className="flex items-center justify-between mb-2 flex-shrink-0">
              <p className="text-[12px] font-semibold">Local Port Scan</p>
              <Button
                size="sm"
                variant="default"
                onClick={scan}
                disabled={isScanning}
                className="h-7 text-xs"
              >
                <Search className="h-3 w-3" /> {isScanning ? "Scanning…" : "Scan"}
              </Button>
            </div>
            <ScrollArea className="flex-1 min-h-0">
              <div className="space-y-2 pr-1">
                {detected.length === 0 ? (
                  <p className="text-center text-[11px] text-muted-foreground italic py-6">
                    {isScanning ? "Searching for endpoints…" : "No active server instances detected."}
                  </p>
                ) : (
                  detected.map((inst) => {
                    const isActive =
                      routedInstance?.host === inst.hostname &&
                      routedInstance.port === inst.port;
                    return (
                      <InstanceCard
                        key={`${inst.hostname}:${inst.port}`}
                        instance={inst}
                        active={isActive}
                        onRoute={() =>
                          setRoute(
                            inst.hostname,
                            inst.port,
                            inst.models[0] ?? null
                          )
                        }
                        routedModel={routedModel}
                        onPickModel={(model) => {
                          setRoutedInstance({
                            host: inst.hostname,
                            port: inst.port,
                          });
                          setRoutedModel(model);
                        }}
                      />
                    );
                  })
                )}
              </div>
            </ScrollArea>
          </div>
        </div>

        <div className="flex-1 min-w-[320px] flex flex-col gap-2">
          <p className="text-[12px] font-semibold text-muted-foreground">
            Target overrides per Tool & Subagent
          </p>
          {config &&
            OVERRIDE_SCOPES.map(({ id, label }) => (
              <OverrideCard key={id} scope={id} label={label} config={config} />
            ))}
        </div>
      </div>
    </div>
  );
}

function InstanceCard({
  instance: inst,
  active,
  onRoute,
  routedModel,
  onPickModel,
}: {
  instance: LlamaInstance;
  active: boolean;
  onRoute: () => void;
  routedModel: string | null;
  onPickModel: (model: string) => void;
}) {
  return (
    <div className="rounded-md border border-border/40 bg-muted/20 p-2 space-y-1.5">
      <div className="flex items-center justify-between gap-2">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <span className="font-semibold text-[11px] truncate">
              http://{inst.hostname}:{inst.port}
            </span>
            <Badge variant="success" className="text-[8px] py-0">
              Online
            </Badge>
          </div>
          <p className="text-[10px] text-muted-foreground truncate">
            {inst.models.join(", ") || "(no models listed)"}
          </p>
        </div>
        <Button
          size="sm"
          variant={active ? "secondary" : "default"}
          onClick={onRoute}
          disabled={active}
          className="h-6 text-[10px] flex-shrink-0"
        >
          {active ? "Routed" : "Route"}
        </Button>
      </div>
      {inst.models.length > 0 && (
        <div className="flex items-center justify-between gap-2 pt-1 border-t border-dashed border-border/40">
          <div className="flex items-center gap-1.5 min-w-0">
            <span className="text-[10px] text-muted-foreground">Model:</span>
            <select
              className="h-6 text-[10px] bg-transparent border border-input rounded px-1 no-drag"
              value={active ? routedModel ?? "" : ""}
              onChange={(e) => onPickModel(e.target.value)}
            >
              <option value="">— Select —</option>
              {inst.models.map((m) => (
                <option key={m} value={m}>
                  {m}
                </option>
              ))}
            </select>
          </div>
          {active && (
            <span className="text-[10px] text-primary font-semibold flex items-center gap-1">
              <Link2 className="h-2.5 w-2.5" /> Active
            </span>
          )}
        </div>
      )}
    </div>
  );
}

function OverrideCard({
  scope,
  label,
  config,
}: {
  scope: ToolScope;
  label: string;
  config: ReturnType<typeof useStore.getState>["config"];
}) {
  const override: ModelOverride = (config as Record<string, ModelOverride>)[
    `override_${scope}`
  ] ?? { enabled: false, host: config?.host ?? "127.0.0.1", port: config?.port ?? 8080, model: "" };

  const setEnabled = async (enabled: boolean) => {
    await patchOverride(scope, { enabled });
  };
  const setHost = async (host: string) => {
    await patchOverride(scope, { host });
  };
  const setPort = async (port: number) => {
    await patchOverride(scope, { port });
  };
  const setModel = async (model: string) => {
    await patchOverride(scope, { model });
  };

  return (
    <div className="rounded-md border border-border/40 p-2.5 space-y-2">
      <div className="flex items-center justify-between gap-2">
        <span className="text-[11px] font-semibold truncate">{label}</span>
        <label className="flex items-center gap-1.5 cursor-pointer no-drag">
          <span className="text-[10px] text-muted-foreground">Override</span>
          <Switch
            checked={override.enabled}
            onCheckedChange={setEnabled}
          />
        </label>
      </div>
      {override.enabled && (
        <div className="grid grid-cols-[2fr_1fr_2fr] gap-1.5">
          <Input
            value={override.host}
            onChange={(e) => setHost(e.target.value)}
            placeholder="Host"
            className="h-7 text-[10px]"
          />
          <Input
            type="number"
            value={override.port}
            onChange={(e) => setPort(Number(e.target.value))}
            placeholder="Port"
            className="h-7 text-[10px]"
          />
          <Input
            value={override.model}
            onChange={(e) => setModel(e.target.value)}
            placeholder="Model override"
            className="h-7 text-[10px]"
          />
        </div>
      )}
    </div>
  );
}
