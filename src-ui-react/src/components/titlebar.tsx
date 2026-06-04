import { ipc } from "@/lib/ipc";
import { useStore } from "@/store/app-store";
import { cn } from "@/lib/utils";

export function Titlebar() {
  const serverOnline = useStore((s) => s.serverOnline);

  return (
    <div
      className="drag-region flex h-10 items-center justify-between border-b border-border/40 px-3 flex-shrink-0 select-none"
      style={{ background: "hsl(var(--sidebar) / 0.6)" }}
    >
      {/* Left: logo + title */}
      <div className="flex items-center gap-2.5 no-drag">
        <span className="text-base">🦙</span>
        <span className="text-sm font-semibold text-foreground/90 tracking-tight">
          llama-manager
        </span>
        <div
          className={cn(
            "h-1.5 w-1.5 rounded-full transition-colors",
            serverOnline ? "bg-emerald-400" : "bg-muted-foreground/40"
          )}
        />
      </div>

      {/* Right: window controls */}
      <div className="flex items-center gap-1 no-drag">
        <WinBtn
          onClick={() => ipc.winMinimize()}
          className="hover:bg-muted"
          title="Minimise"
        >
          <MinimiseIcon />
        </WinBtn>
        <WinBtn
          onClick={() => ipc.winToggleMaximize()}
          className="hover:bg-muted"
          title="Maximise"
        >
          <MaximiseIcon />
        </WinBtn>
        <WinBtn
          onClick={() => ipc.winClose()}
          className="hover:bg-destructive hover:text-destructive-foreground"
          title="Close"
        >
          <CloseIcon />
        </WinBtn>
      </div>
    </div>
  );
}

function WinBtn({
  onClick,
  className,
  title,
  children,
}: {
  onClick: () => void;
  className?: string;
  title?: string;
  children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      title={title}
      className={cn(
        "flex h-6 w-6 items-center justify-center rounded transition-colors text-muted-foreground",
        className
      )}
    >
      {children}
    </button>
  );
}

function MinimiseIcon() {
  return (
    <svg width="10" height="10" viewBox="0 0 10 1" fill="none">
      <rect width="10" height="1" fill="currentColor" />
    </svg>
  );
}

function MaximiseIcon() {
  return (
    <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
      <rect x="0.5" y="0.5" width="9" height="9" stroke="currentColor" strokeWidth="1" fill="none" />
    </svg>
  );
}

function CloseIcon() {
  return (
    <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
      <line x1="1" y1="1" x2="9" y2="9" stroke="currentColor" strokeWidth="1.2" />
      <line x1="9" y1="1" x2="1" y2="9" stroke="currentColor" strokeWidth="1.2" />
    </svg>
  );
}
