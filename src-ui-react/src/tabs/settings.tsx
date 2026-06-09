import { useState, useEffect } from "react";
import { Save, FolderOpen, RefreshCw, Palette, Server, Cpu, Sliders } from "lucide-react";
import { useStore } from "@/store/app-store";
import { ipc, type ServerConfig } from "@/lib/ipc";
import { applyTheme } from "@/lib/theme";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Slider } from "@/components/ui/slider";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

type Section = "server" | "model" | "appearance" | "advanced";

const SECTIONS: { id: Section; label: string; icon: React.ComponentType<{ className?: string }> }[] = [
  { id: "server", label: "Server", icon: Server },
  { id: "model", label: "Model", icon: Cpu },
  { id: "appearance", label: "Appearance", icon: Palette },
  { id: "advanced", label: "Advanced", icon: Sliders },
];

export function SettingsTab() {
  const { config, setConfig } = useStore();
  const [local, setLocal] = useState<ServerConfig | null>(null);
  const [section, setSection] = useState<Section>("server");
  const [saved, setSaved] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (config && !local) setLocal({ ...config });
  }, [config]);

  if (!local) return (
    <div className="flex items-center justify-center h-full text-muted-foreground text-sm">Loading config…</div>
  );

  const set = <K extends keyof ServerConfig>(key: K, val: ServerConfig[K]) =>
    setLocal((prev) => prev ? { ...prev, [key]: val } : prev);

  const save = async () => {
    if (!local) return;
    setSaving(true);
    try {
      await ipc.updateConfig(local);
      setConfig(local);
      applyTheme(local);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error(e);
    } finally {
      setSaving(false);
    }
  };

  const pickPath = async (key: keyof ServerConfig, dir = false) => {
    const path = await ipc.pickPath(dir);
    if (path) set(key, path as ServerConfig[typeof key]);
  };

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-border/40 flex-shrink-0" style={{ background: "hsl(var(--card) / 0.4)" }}>
        <span className="text-sm font-medium">Settings</span>
        <Button size="sm" onClick={save} disabled={saving}>
          {saving ? <RefreshCw className="h-3.5 w-3.5 animate-spin" /> : <Save className="h-3.5 w-3.5" />}
          {saved ? "Saved!" : "Save"}
        </Button>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {/* Section nav */}
        <div className="w-36 flex-shrink-0 border-r border-border/30 py-2 px-1">
          {SECTIONS.map((s) => {
            const Icon = s.icon;
            return (
              <button
                key={s.id}
                onClick={() => setSection(s.id)}
                className={cn(
                  "flex w-full items-center gap-2 rounded px-2.5 py-2 text-sm transition-colors no-drag",
                  section === s.id ? "bg-primary/15 text-foreground font-medium" : "text-muted-foreground hover:bg-accent hover:text-foreground"
                )}
              >
                <Icon className="h-3.5 w-3.5" />
                {s.label}
              </button>
            );
          })}
        </div>

        {/* Section content */}
        <ScrollArea className="flex-1">
          <div className="p-4 space-y-5 max-w-xl">
            {section === "server" && <ServerSection local={local} set={set} pickPath={pickPath} />}
            {section === "model" && <ModelSection local={local} set={set} pickPath={pickPath} />}
            {section === "appearance" && <AppearanceSection local={local} set={set} />}
            {section === "advanced" && <AdvancedSection local={local} set={set} />}
          </div>
        </ScrollArea>
      </div>
    </div>
  );
}

function FieldRow({ label, hint, children }: { label: string; hint?: string; children: React.ReactNode }) {
  return (
    <div className="space-y-1.5">
      <Label className="text-xs">{label}</Label>
      {children}
      {hint && <p className="text-[10px] text-muted-foreground">{hint}</p>}
    </div>
  );
}

function SwitchRow({ label, hint, checked, onCheckedChange }: { label: string; hint?: string; checked: boolean; onCheckedChange: (v: boolean) => void }) {
  return (
    <div className="flex items-center justify-between rounded-md border border-border/40 px-3 py-2">
      <div>
        <p className="text-sm">{label}</p>
        {hint && <p className="text-[10px] text-muted-foreground">{hint}</p>}
      </div>
      <Switch checked={checked} onCheckedChange={onCheckedChange} />
    </div>
  );
}

function PathInput({ value, onChange, onPick, dir = false }: { value: string; onChange: (v: string) => void; onPick: (dir?: boolean) => void; dir?: boolean }) {
  return (
    <div className="flex gap-2">
      <Input value={value} onChange={(e) => onChange(e.target.value)} className="text-xs font-mono flex-1" />
      <Button size="icon-sm" variant="outline" onClick={() => onPick(dir)} title="Browse">
        <FolderOpen className="h-3.5 w-3.5" />
      </Button>
    </div>
  );
}

function ServerSection({ local, set, pickPath }: { local: ServerConfig; set: <K extends keyof ServerConfig>(k: K, v: ServerConfig[K]) => void; pickPath: (k: keyof ServerConfig, dir?: boolean) => Promise<void> }) {
  return (
    <>
      <FieldRow label="llama-server executable">
        <PathInput value={local.exe_path} onChange={(v) => set("exe_path", v)} onPick={() => pickPath("exe_path")} />
      </FieldRow>
      <FieldRow label="Host">
        <Input value={local.host} onChange={(e) => set("host", e.target.value)} className="text-xs" />
      </FieldRow>
      <FieldRow label="Port">
        <Input type="number" value={local.port} onChange={(e) => set("port", Number(e.target.value))} className="text-xs w-28" />
      </FieldRow>
      <FieldRow label="SearXNG URL" hint="Used for web search tool">
        <Input value={local.searxng_url} onChange={(e) => set("searxng_url", e.target.value)} className="text-xs font-mono" placeholder="http://localhost:8888" />
      </FieldRow>
      <SwitchRow label="Flash Attention (-fa)" hint="Faster inference on supported hardware" checked={local.flash_attn} onCheckedChange={(v) => set("flash_attn", v)} />
      <SwitchRow label="Continuous Batching (-cb)" hint="Better throughput with multiple users" checked={local.cont_batching} onCheckedChange={(v) => set("cont_batching", v)} />
    </>
  );
}

function ModelSection({ local, set, pickPath }: { local: ServerConfig; set: <K extends keyof ServerConfig>(k: K, v: ServerConfig[K]) => void; pickPath: (k: keyof ServerConfig, dir?: boolean) => Promise<void> }) {
  return (
    <>
      <FieldRow label="Model path (.gguf)">
        <PathInput value={local.model_path} onChange={(v) => set("model_path", v)} onPick={() => pickPath("model_path")} />
      </FieldRow>
      <FieldRow label="Model alias" hint="Display name and identifier sent to the API">
        <Input value={local.model_alias} onChange={(e) => set("model_alias", e.target.value)} className="text-xs" />
      </FieldRow>
      <FieldRow label={`Context size: ${local.ctx_size?.toLocaleString()}`}>
        <Slider value={[local.ctx_size ?? 4096]} min={512} max={131072} step={512} onValueChange={([v]) => set("ctx_size", v)} />
      </FieldRow>
      <FieldRow label={`GPU layers: ${local.gpu_layers}`} hint="Layers offloaded to GPU; -1 = all">
        <Slider value={[local.gpu_layers ?? 0]} min={-1} max={200} step={1} onValueChange={([v]) => set("gpu_layers", v)} />
      </FieldRow>
      <FieldRow label={`CPU threads: ${local.threads ?? 0}`} hint="0 = auto">
        <Slider value={[local.threads ?? 0]} min={0} max={64} step={1} onValueChange={([v]) => set("threads", v)} />
      </FieldRow>
      <FieldRow label={`Temperature: ${(local.temperature ?? 0.7).toFixed(2)}`}>
        <Slider value={[local.temperature ?? 0.7]} min={0} max={2} step={0.05} onValueChange={([v]) => set("temperature", v)} />
      </FieldRow>
      <FieldRow label="System prompt">
        <textarea
          value={local.system_prompt ?? ""}
          onChange={(e) => set("system_prompt", e.target.value)}
          rows={4}
          className="flex w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring resize-none no-drag"
          placeholder="You are a helpful assistant…"
        />
      </FieldRow>
    </>
  );
}

function AppearanceSection({ local, set }: { local: ServerConfig; set: <K extends keyof ServerConfig>(k: K, v: ServerConfig[K]) => void }) {
  return (
    <>
      <SwitchRow label="Dark mode" checked={local.dark_mode} onCheckedChange={(v) => set("dark_mode", v)} />
      <FieldRow label={`Window opacity: ${Math.round((1 - (local.ui_transparency ?? 0.1)) * 100)}%`} hint="Transparency of the window background">
        <Slider value={[1 - (local.ui_transparency ?? 0.1)]} min={0.1} max={1} step={0.01} onValueChange={([v]) => set("ui_transparency", 1 - v)} />
      </FieldRow>
      <SwitchRow label="Background blur" hint="Frosted glass effect (requires compositor)" checked={local.ui_blur} onCheckedChange={(v) => set("ui_blur", v)} />
      {local.ui_blur && (
        <FieldRow label={`Blur intensity: ${local.ui_blur_intensity ?? 28}px`}>
          <Slider value={[local.ui_blur_intensity ?? 28]} min={0} max={60} step={2} onValueChange={([v]) => set("ui_blur_intensity", v)} />
        </FieldRow>
      )}
      <div className="grid grid-cols-2 gap-3">
        <ColorField label="Background" value={local.ui_background_color} onChange={(v) => set("ui_background_color", v)} />
        <ColorField label="Text" value={local.ui_text_color} onChange={(v) => set("ui_text_color", v)} />
        <ColorField label="Accent" value={local.ui_accent_color} onChange={(v) => set("ui_accent_color", v)} />
        <ColorField label="Border" value={local.ui_border_color} onChange={(v) => set("ui_border_color", v)} />
      </div>
      <div className="grid grid-cols-2 gap-3 mt-2">
        <ColorField label="Card" value={local.ui_card_bg} onChange={(v) => set("ui_card_bg", v)} />
        <ColorField label="Card text" value={local.ui_card_text} onChange={(v) => set("ui_card_text", v)} />
        <ColorField label="Button" value={local.ui_button_bg} onChange={(v) => set("ui_button_bg", v)} />
        <ColorField label="Button text" value={local.ui_button_text} onChange={(v) => set("ui_button_text", v)} />
      </div>
      <div className="mt-2">
        <p className="text-xs font-medium mb-2 text-muted-foreground">Light mode colors</p>
        <div className="grid grid-cols-2 gap-3">
          <ColorField label="Background" value={local.ui_light_background_color} onChange={(v) => set("ui_light_background_color", v)} />
          <ColorField label="Text" value={local.ui_light_text_color} onChange={(v) => set("ui_light_text_color", v)} />
          <ColorField label="Accent" value={local.ui_light_accent_color} onChange={(v) => set("ui_light_accent_color", v)} />
          <ColorField label="Border" value={local.ui_light_border_color} onChange={(v) => set("ui_light_border_color", v)} />
        </div>
        <div className="grid grid-cols-2 gap-3 mt-2">
          <ColorField label="Card" value={local.ui_light_card_bg} onChange={(v) => set("ui_light_card_bg", v)} />
          <ColorField label="Card text" value={local.ui_light_card_text} onChange={(v) => set("ui_light_card_text", v)} />
          <ColorField label="Button" value={local.ui_light_button_bg} onChange={(v) => set("ui_light_button_bg", v)} />
          <ColorField label="Button text" value={local.ui_light_button_text} onChange={(v) => set("ui_light_button_text", v)} />
        </div>
      </div>
      <FieldRow label="Font family">
        <Input value={local.ui_font_family ?? "Inter"} onChange={(e) => set("ui_font_family", e.target.value)} className="text-xs" placeholder="Inter" />
      </FieldRow>
      <div className="grid grid-cols-2 gap-3">
        <FieldRow label="Radius SM">
          <Input value={local.ui_radius_sm ?? "4px"} onChange={(e) => set("ui_radius_sm", e.target.value)} className="text-xs" />
        </FieldRow>
        <FieldRow label="Radius MD">
          <Input value={local.ui_radius_md ?? "6px"} onChange={(e) => set("ui_radius_md", e.target.value)} className="text-xs" />
        </FieldRow>
        <FieldRow label="Radius LG">
          <Input value={local.ui_radius_lg ?? "8px"} onChange={(e) => set("ui_radius_lg", e.target.value)} className="text-xs" />
        </FieldRow>
        <FieldRow label="Radius XL">
          <Input value={local.ui_radius_xl ?? "12px"} onChange={(e) => set("ui_radius_xl", e.target.value)} className="text-xs" />
        </FieldRow>
      </div>
    </>
  );
}

function ColorField({ label, value, onChange }: { label: string; value: string; onChange: (v: string) => void }) {
  return (
    <div className="space-y-1">
      <Label className="text-[10px]">{label}</Label>
      <div className="flex items-center gap-2">
        <input type="color" value={value?.startsWith("#") ? value : "#6366f1"} onChange={(e) => onChange(e.target.value)} className="h-7 w-8 cursor-pointer rounded border border-input bg-transparent p-0.5 no-drag" />
        <Input value={value} onChange={(e) => onChange(e.target.value)} className="text-xs font-mono flex-1 h-7" />
      </div>
    </div>
  );
}

function AdvancedSection({ local, set }: { local: ServerConfig; set: <K extends keyof ServerConfig>(k: K, v: ServerConfig[K]) => void }) {
  return (
    <>
      <FieldRow label={`Batch size: ${local.batch_size ?? 512}`}>
        <Slider value={[local.batch_size ?? 512]} min={64} max={4096} step={64} onValueChange={([v]) => set("batch_size", v)} />
      </FieldRow>
      <FieldRow label={`Micro-batch size: ${local.ubatch_size ?? 512}`} hint="Should not exceed batch size">
        <Slider value={[local.ubatch_size ?? 512]} min={64} max={4096} step={64} onValueChange={([v]) => set("ubatch_size", v)} />
      </FieldRow>
      <FieldRow label={`Parallel slots: ${local.parallel ?? 1}`} hint="Concurrent request slots">
        <Slider value={[local.parallel ?? 1]} min={1} max={16} step={1} onValueChange={([v]) => set("parallel", v)} />
      </FieldRow>
      <FieldRow label="HuggingFace repo" hint="e.g. TheBloke/Mixtral-8x7B-v0.1-GGUF">
        <Input value={local.hf_repo ?? ""} onChange={(e) => set("hf_repo", e.target.value)} className="text-xs font-mono" />
      </FieldRow>
    </>
  );
}
