import { useEffect, useState } from "react";
import { Play, Gauge, Eye, EyeOff, Vote, Loader2 } from "lucide-react";
import { ipc, type BenchmarkOutput } from "@/lib/ipc";
import { resolveTarget } from "@/lib/target";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";

type Mode = "bench" | "eval";

export function CompareTab() {
  const [mode, setMode] = useState<Mode>("bench");

  // Bench fields
  const [benchUrl, setBenchUrl] = useState("");
  const [benchModel, setBenchModel] = useState("");
  const [benchPp, setBenchPp] = useState("512 1024 2048");
  const [benchTg, setBenchTg] = useState("128 256");
  const [benchRuns, setBenchRuns] = useState("5");
  const [benchOutput, setBenchOutput] = useState<BenchmarkOutput | null>(null);
  const [isBenching, setIsBenching] = useState(false);
  const [benchError, setBenchError] = useState("");

  // Eval fields
  const [evalUrl, setEvalUrl] = useState("");
  const [evalModelA, setEvalModelA] = useState("");
  const [evalModelB, setEvalModelB] = useState("");
  const [evalPrompt, setEvalPrompt] = useState("");
  const [evalResA, setEvalResA] = useState("");
  const [evalResB, setEvalResB] = useState("");
  const [isEvaluating, setIsEvaluating] = useState(false);
  const [reveal, setReveal] = useState(false);
  const [voteStatus, setVoteStatus] = useState("");

  useEffect(() => {
    const { host, port, model } = resolveTarget("compare");
    const url = `http://${host}:${port}`;
    setBenchUrl(url);
    setBenchModel(model);
    setEvalUrl(url);
    setEvalModelA(model);
  }, []);

  const runBenchmark = async () => {
    setIsBenching(true);
    setBenchError("");
    setBenchOutput(null);
    try {
      const out = await ipc.compareRunBench(
        benchUrl,
        benchModel,
        benchPp,
        benchTg,
        String(Number(benchRuns) || 5)
      );
      setBenchOutput(out);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setBenchError(msg);
    } finally {
      setIsBenching(false);
    }
  };

  const runEval = async () => {
    setIsEvaluating(true);
    setReveal(false);
    setVoteStatus("");
    setEvalResA("Running Model A evaluation…");
    setEvalResB("Running Model B evaluation…");
    const [resA, resB] = await Promise.allSettled([
      ipc.compareRunEval(evalUrl, evalModelA, evalPrompt),
      ipc.compareRunEval(evalUrl, evalModelB, evalPrompt),
    ]);
    setEvalResA(
      resA.status === "fulfilled" ? resA.value : `Error: ${resA.reason}`
    );
    setEvalResB(
      resB.status === "fulfilled" ? resB.value : `Error: ${resB.reason}`
    );
    setIsEvaluating(false);
  };

  return (
    <div className="flex flex-col h-full overflow-hidden p-3 gap-3">
      <div>
        <h2 className="text-sm font-semibold">Compare Models</h2>
        <p className="text-[10px] text-muted-foreground">
          Execute llama-benchy performance suites or blind side-by-side prompt testing.
        </p>
      </div>

      <div className="flex glass-card p-1 rounded-full w-fit">
        <ModePill current={mode === "bench"} onClick={() => setMode("bench")} icon={<Gauge className="h-3 w-3" />}>
          Performance Benchmark
        </ModePill>
        <ModePill current={mode === "eval"} onClick={() => setMode("eval")} icon={<Vote className="h-3 w-3" />}>
          Blind Prompt Test
        </ModePill>
      </div>

      <div className="flex-1 min-h-0 overflow-y-auto">
        {mode === "bench" ? (
          <BenchMode
            url={benchUrl} setUrl={setBenchUrl}
            model={benchModel} setModel={setBenchModel}
            pp={benchPp} setPp={setBenchPp}
            tg={benchTg} setTg={setBenchTg}
            runs={benchRuns} setRuns={setBenchRuns}
            output={benchOutput}
            isRunning={isBenching}
            error={benchError}
            onRun={runBenchmark}
          />
        ) : (
          <EvalMode
            url={evalUrl} setUrl={setEvalUrl}
            modelA={evalModelA} setModelA={setEvalModelA}
            modelB={evalModelB} setModelB={setEvalModelB}
            prompt={evalPrompt} setPrompt={setEvalPrompt}
            resA={evalResA} resB={evalResB}
            isRunning={isEvaluating}
            reveal={reveal} setReveal={setReveal}
            voteStatus={voteStatus} setVoteStatus={setVoteStatus}
            onRun={runEval}
          />
        )}
      </div>
    </div>
  );
}

function ModePill({
  current, onClick, icon, children,
}: { current: boolean; onClick: () => void; icon: React.ReactNode; children: React.ReactNode }) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "flex items-center gap-1.5 px-3 h-7 rounded-full text-[11.5px] font-semibold transition-colors",
        current ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:bg-muted/40"
      )}
    >
      {icon}
      {children}
    </button>
  );
}

function BenchMode({
  url, setUrl, model, setModel, pp, setPp, tg, setTg, runs, setRuns,
  output, isRunning, error, onRun,
}: {
  url: string; setUrl: (s: string) => void;
  model: string; setModel: (s: string) => void;
  pp: string; setPp: (s: string) => void;
  tg: string; setTg: (s: string) => void;
  runs: string; setRuns: (s: string) => void;
  output: BenchmarkOutput | null;
  isRunning: boolean;
  error: string;
  onRun: () => void;
}) {
  return (
    <div className="space-y-3">
      <div className="glass-card p-3 space-y-3">
        <p className="text-[11px] font-semibold">Benchy Config</p>
        <div className="grid grid-cols-[2fr_1fr] gap-2">
          <Field label="Target Endpoint URL">
            <Input value={url} onChange={(e) => setUrl(e.target.value)} className="h-7 text-[11px]" />
          </Field>
          <Field label="Model Reference Name">
            <Input value={model} onChange={(e) => setModel(e.target.value)} placeholder="e.g. meta-llama-3" className="h-7 text-[11px]" />
          </Field>
        </div>
        <div className="grid grid-cols-3 gap-2">
          <Field label="Prompt Processing (PP)">
            <Input value={pp} onChange={(e) => setPp(e.target.value)} className="h-7 text-[11px]" />
          </Field>
          <Field label="Token Generation (TG)">
            <Input value={tg} onChange={(e) => setTg(e.target.value)} className="h-7 text-[11px]" />
          </Field>
          <Field label="Bench Runs">
            <Input type="number" value={runs} onChange={(e) => setRuns(e.target.value)} className="h-7 text-[11px]" />
          </Field>
        </div>
        <Button onClick={onRun} disabled={isRunning} size="sm" className="h-8">
          {isRunning ? (
            <><Loader2 className="h-3.5 w-3.5 animate-spin" /> Running Benchmark…</>
          ) : (
            <><Play className="h-3.5 w-3.5" /> Run Performance Suite</>
          )}
        </Button>
      </div>

      {error && (
        <div className="glass-card p-2.5 text-[11px] border-amber-500/40 text-amber-400">
          {error}
        </div>
      )}

      {output && (
        <>
          <div className="glass-card p-3 space-y-2">
            <p className="text-[11px] font-semibold">Benchmark Speeds</p>
            <table className="w-full text-[12px] border-collapse">
              <thead>
                <tr className="text-muted-foreground text-[10px] uppercase tracking-wider border-b border-border/40">
                  <th className="text-left py-1.5">PP Tokens</th>
                  <th className="text-left py-1.5">TG Tokens</th>
                  <th className="text-left py-1.5">PP Speed</th>
                  <th className="text-left py-1.5">TG Speed</th>
                </tr>
              </thead>
              <tbody>
                {output.results.map((r, i) => (
                  <tr key={i} className="border-b border-border/20">
                    <td className="py-1.5">{r.pp}</td>
                    <td className="py-1.5">{r.tg}</td>
                    <td className="py-1.5 font-semibold text-primary">{r.pp_speed} t/s</td>
                    <td className="py-1.5 font-semibold text-primary">{r.tg_speed} t/s</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <div className="glass-card p-3 space-y-1.5">
            <p className="text-[11px] font-semibold">Raw Terminal Output</p>
            <pre className="text-[11px] font-mono bg-muted/30 rounded p-2 max-h-[280px] overflow-y-auto whitespace-pre-wrap">
              {output.raw}
            </pre>
          </div>
        </>
      )}
    </div>
  );
}

function EvalMode({
  url, setUrl, modelA, setModelA, modelB, setModelB, prompt, setPrompt,
  resA, resB, isRunning, reveal, setReveal, voteStatus, setVoteStatus, onRun,
}: {
  url: string; setUrl: (s: string) => void;
  modelA: string; setModelA: (s: string) => void;
  modelB: string; setModelB: (s: string) => void;
  prompt: string; setPrompt: (s: string) => void;
  resA: string; resB: string;
  isRunning: boolean;
  reveal: boolean; setReveal: (b: boolean) => void;
  voteStatus: string; setVoteStatus: (s: string) => void;
  onRun: () => void;
}) {
  return (
    <div className="space-y-3">
      <div className="glass-card p-3 space-y-3">
        <p className="text-[11px] font-semibold">Blind Prompt Configuration</p>
        <Field label="Endpoint API URL">
          <Input value={url} onChange={(e) => setUrl(e.target.value)} className="h-7 text-[11px]" />
        </Field>
        <div className="grid grid-cols-2 gap-2">
          <Field label="Model A Name">
            <Input value={modelA} onChange={(e) => setModelA(e.target.value)} placeholder="e.g. meta-llama-3" className="h-7 text-[11px]" />
          </Field>
          <Field label="Model B Name">
            <Input value={modelB} onChange={(e) => setModelB(e.target.value)} placeholder="e.g. gemma-7b" className="h-7 text-[11px]" />
          </Field>
        </div>
        <Field label="Evaluation Prompt">
          <Textarea
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            placeholder="Enter prompt to execute side-by-side…"
            className="min-h-[100px] text-[12.5px]"
          />
        </Field>
        <Button onClick={onRun} disabled={isRunning} size="sm" className="h-8">
          {isRunning ? (
            <><Loader2 className="h-3.5 w-3.5 animate-spin" /> Running Evaluations…</>
          ) : (
            <><Play className="h-3.5 w-3.5" /> Run Blind Test</>
          )}
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        <OutputPane title="Model 1 Output" text={resA} modelName={modelA} reveal={reveal} />
        <OutputPane title="Model 2 Output" text={resB} modelName={modelB} reveal={reveal} />
      </div>

      <div className="glass-card p-3 space-y-2">
        <p className="text-[11px] font-semibold">Judge responses</p>
        <div className="flex flex-wrap gap-2">
          <Button size="sm" variant="outline" disabled={isRunning} onClick={() => setVoteStatus("Voted for Model 1!")} className="h-7 text-[11px]">
            Vote Model 1
          </Button>
          <Button size="sm" variant="outline" disabled={isRunning} onClick={() => setVoteStatus("Voted for Model 2!")} className="h-7 text-[11px]">
            Vote Model 2
          </Button>
          <Button size="sm" onClick={() => setReveal(true)} className="h-7 text-[11px]">
            {reveal ? <EyeOff className="h-3 w-3" /> : <Eye className="h-3 w-3" />}
            {reveal ? "Hide Model Names" : "Reveal Model Names"}
          </Button>
        </div>
        {voteStatus && (
          <div className="text-[11.5px] font-semibold text-emerald-400">
            {voteStatus}
          </div>
        )}
      </div>
    </div>
  );
}

function OutputPane({
  title, text, modelName, reveal,
}: { title: string; text: string; modelName: string; reveal: boolean }) {
  return (
    <div className="glass-card p-3 space-y-2">
      <p className="text-[11px] font-semibold">{title}</p>
      <ScrollArea className="h-[320px] bg-muted/30 rounded-md p-2">
        <p className="text-[12.5px] whitespace-pre-wrap break-words font-sans">
          {text || <span className="text-muted-foreground italic">(no output yet)</span>}
        </p>
      </ScrollArea>
      {reveal && (
        <p className="text-[12px] font-semibold">Model Name: {modelName}</p>
      )}
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="block space-y-1">
      <span className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
        {label}
      </span>
      {children}
    </label>
  );
}
