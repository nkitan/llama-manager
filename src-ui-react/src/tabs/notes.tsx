import { useState, useEffect, useCallback } from "react";
import { Plus, Trash2, Pin, Save } from "lucide-react";
import { ipc, type Note, type NotesStore } from "@/lib/ipc";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

export function NotesTab() {
  const [store, setStore] = useState<NotesStore>({ notes: [] });
  const [scope, setScope] = useState<"project" | "global">("project");
  const [activeIdx, setActiveIdx] = useState(0);
  const [draft, setDraft] = useState("");
  const [draftName, setDraftName] = useState("");
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    ipc.notesStoreGet(scope).then((s) => {
      setStore(s);
      setActiveIdx(0);
      const first = s.notes[0];
      setDraft(first?.content ?? "");
      setDraftName(first?.name ?? "");
      setDirty(false);
    });
  }, [scope]);

  useEffect(() => {
    const note = store.notes[activeIdx];
    if (note) {
      setDraft(note.content);
      setDraftName(note.name);
      setDirty(false);
    }
  }, [activeIdx]);

  const save = useCallback(async () => {
    setSaving(true);
    const updated = store.notes.map((n, i) =>
      i === activeIdx ? { ...n, name: draftName, content: draft } : n
    );
    const newStore = { notes: updated };
    await ipc.notesStoreSet(scope, newStore).catch(() => {});
    setStore(newStore);
    setDirty(false);
    setSaving(false);
  }, [store, activeIdx, draftName, draft, scope]);

  const addNote = async () => {
    const name = `Note ${store.notes.length + 1}`;
    const newNote: Note = { name, content: "", tags: [], pinned: false };
    const newStore = { notes: [...store.notes, newNote] };
    await ipc.notesStoreSet(scope, newStore).catch(() => {});
    setStore(newStore);
    setActiveIdx(newStore.notes.length - 1);
    setDraft("");
    setDraftName(name);
    setDirty(false);
  };

  const deleteNote = async (idx: number) => {
    if (store.notes.length <= 1) return;
    const updated = store.notes.filter((_, i) => i !== idx);
    const newStore = { notes: updated };
    await ipc.notesStoreSet(scope, newStore).catch(() => {});
    setStore(newStore);
    const newIdx = Math.min(idx, updated.length - 1);
    setActiveIdx(newIdx);
    setDraft(updated[newIdx]?.content ?? "");
    setDraftName(updated[newIdx]?.name ?? "");
    setDirty(false);
  };

  const togglePin = async (idx: number) => {
    const updated = store.notes.map((n, i) => i === idx ? { ...n, pinned: !n.pinned } : n);
    const newStore = { notes: updated };
    await ipc.notesStoreSet(scope, newStore).catch(() => {});
    setStore(newStore);
  };

  return (
    <div className="flex h-full overflow-hidden">
      {/* Notes list sidebar */}
      <div className="w-44 flex-shrink-0 flex flex-col border-r border-border/30">
        {/* Scope toggle */}
        <div className="flex border-b border-border/30 flex-shrink-0">
          {(["project", "global"] as const).map((s) => (
            <button
              key={s}
              onClick={() => setScope(s)}
              className={cn(
                "flex-1 py-2 text-[11px] font-medium transition-colors no-drag",
                scope === s ? "text-foreground border-b-2 border-primary" : "text-muted-foreground hover:text-foreground"
              )}
            >
              {s === "project" ? "Project" : "Global"}
            </button>
          ))}
        </div>

        <ScrollArea className="flex-1">
          <div className="p-1.5 space-y-0.5">
            {store.notes
              .map((n, i) => ({ ...n, idx: i }))
              .sort((a, b) => (b.pinned ? 1 : 0) - (a.pinned ? 1 : 0))
              .map(({ idx, name, content, pinned }) => (
                <button
                  key={idx}
                  onClick={() => { if (dirty) save(); setActiveIdx(idx); }}
                  className={cn(
                    "group flex w-full flex-col items-start rounded px-2 py-1.5 text-left transition-colors no-drag",
                    activeIdx === idx ? "bg-primary/15 text-foreground" : "text-muted-foreground hover:bg-accent hover:text-foreground"
                  )}
                >
                  <div className="flex items-center gap-1 w-full min-w-0">
                    {pinned && <Pin className="h-2.5 w-2.5 flex-shrink-0 text-primary" />}
                    <span className="text-[11px] font-medium truncate">{name}</span>
                  </div>
                  <span className="text-[10px] text-muted-foreground truncate w-full">
                    {content.slice(0, 40) || "Empty"}
                  </span>
                </button>
              ))}
          </div>
        </ScrollArea>

        <div className="p-1.5 border-t border-border/30">
          <Button variant="ghost" size="sm" onClick={addNote} className="w-full text-xs h-7">
            <Plus className="h-3.5 w-3.5" /> New note
          </Button>
        </div>
      </div>

      {/* Editor */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {store.notes.length > 0 ? (
          <>
            {/* Note header */}
            <div className="flex items-center gap-2 px-3 py-2 border-b border-border/30 flex-shrink-0">
              <Input
                value={draftName}
                onChange={(e) => { setDraftName(e.target.value); setDirty(true); }}
                className="flex-1 h-7 text-sm font-medium border-0 bg-transparent shadow-none focus-visible:ring-0 px-0"
                placeholder="Note title"
              />
              {dirty && <Badge variant="warning" className="text-[10px]">Unsaved</Badge>}
              <Button size="icon-sm" variant="ghost" onClick={() => togglePin(activeIdx)} title="Pin">
                <Pin className={cn("h-3.5 w-3.5", store.notes[activeIdx]?.pinned ? "text-primary" : "")} />
              </Button>
              <Button size="icon-sm" variant="ghost" onClick={() => deleteNote(activeIdx)} title="Delete">
                <Trash2 className="h-3.5 w-3.5" />
              </Button>
              <Button size="icon-sm" variant="default" onClick={save} disabled={saving || !dirty}>
                <Save className="h-3.5 w-3.5" />
              </Button>
            </div>

            {/* Textarea */}
            <textarea
              value={draft}
              onChange={(e) => { setDraft(e.target.value); setDirty(true); }}
              onKeyDown={(e) => { if ((e.ctrlKey || e.metaKey) && e.key === "s") { e.preventDefault(); save(); } }}
              className="flex-1 w-full resize-none bg-transparent p-4 text-sm font-mono focus:outline-none placeholder:text-muted-foreground/50 no-drag"
              placeholder="Start writing… (Ctrl+S to save)"
              spellCheck={false}
            />
          </>
        ) : (
          <div className="flex items-center justify-center h-full text-muted-foreground text-sm">
            No notes yet — create one
          </div>
        )}
      </div>
    </div>
  );
}
