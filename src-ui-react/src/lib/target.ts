import { useStore } from "@/store/app-store";
import type { ServerConfig, ModelOverride } from "@/lib/ipc";
import { ipc } from "@/lib/ipc";

/**
 * Resolve the (host, port, model) triple for a given subsystem. Order:
 *   1. Per-tool override (e.g. `override_compare`) if enabled
 *   2. Global routed instance + routed model (set from Instances tab)
 *   3. Local launch-config (config.host / config.port / config.model_alias)
 */
export function resolveTarget(scope: ToolScope): {
  host: string;
  port: number;
  model: string;
} {
  const cfg = useStore.getState().config;
  const routed = useStore.getState().routedInstance;
  const routedModel = useStore.getState().routedModel;
  if (!cfg) return { host: "127.0.0.1", port: 8080, model: "" };

  const override: ModelOverride | undefined = (cfg as ServerConfig & Record<string, ModelOverride>)[
    `override_${scope}`
  ];
  if (override?.enabled) {
    return {
      host: override.host || cfg.host,
      port: override.port || cfg.port,
      model: override.model || routedModel || cfg.model_alias || cfg.model_path || "",
    };
  }
  if (routed) {
    return {
      host: routed.host,
      port: routed.port,
      model: routedModel || cfg.model_alias || cfg.model_path || "",
    };
  }
  return {
    host: cfg.host,
    port: cfg.port,
    model: cfg.model_alias || cfg.model_path || "",
  };
}

export type ToolScope =
  | "planner"
  | "calendar"
  | "memory"
  | "research"
  | "compare";

/** Patch the global config in-place and persist it. */
export async function patchConfig(patch: Partial<ServerConfig>) {
  const cur = useStore.getState().config;
  if (!cur) return;
  const next = { ...cur, ...patch };
  useStore.getState().setConfig(next);
  try {
    await ipc.updateConfig(next);
  } catch (e) {
    console.error("patchConfig save failed:", e);
  }
}

/** Patch a per-tool override in the global config and persist it. */
export async function patchOverride(
  scope: ToolScope,
  patch: Partial<ModelOverride>
) {
  const cur = useStore.getState().config;
  if (!cur) return;
  const key = `override_${scope}` as const;
  const current = (cur as ServerConfig & Record<string, ModelOverride>)[key] ?? {
    enabled: false,
    host: cur.host,
    port: cur.port,
    model: cur.model_alias,
  };
  const next = { ...current, ...patch };
  await patchConfig({ [key]: next } as unknown as Partial<ServerConfig>);
}
