import { useState, useEffect, useRef } from "react";
import { ChevronLeft, ChevronRight, Plus, Trash2, Clock, X, Info, Calendar as CalIcon, List as ListIcon } from "lucide-react";
import { ipc, type CalendarEvent } from "@/lib/ipc";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";

const WEEKDAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const MONTHS = ["January","February","March","April","May","June","July","August","September","October","November","December"];
const COLORS = ["#6366f1", "#8b5cf6", "#10b981", "#ef4444", "#f59e0b"];
const RECURRENCE_LABEL: Record<string, string> = {
  daily:   "🔁 Daily",
  weekly:  "📅 Weekly",
  monthly: "🗓 Monthly",
};

export function CalendarTab() {
  const [events, setEvents] = useState<CalendarEvent[]>([]);
  const [year, setYear] = useState(new Date().getFullYear());
  const [month, setMonth] = useState(new Date().getMonth());
  const [selected, setSelected] = useState<number | null>(null);
  const [showAdd, setShowAdd] = useState(false);
  const [addDate, setAddDate] = useState("");
  const [showInfo, setShowInfo] = useState(false);
  const [view, setView] = useState<"month" | "list">("month");
  const [expanded, setExpanded] = useState<CalendarEvent | null>(null);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    ipc.calendarLoad().then((s) => setEvents(s.events));
    pollRef.current = setInterval(() => {
      ipc.calendarLoad().then((s) => setEvents(s.events)).catch(() => {});
    }, 3000);
    return () => { if (pollRef.current) clearInterval(pollRef.current); };
  }, []);

  const daysInMonth = new Date(year, month + 1, 0).getDate();
  const firstDayRaw = new Date(year, month, 1).getDay(); // 0=Sun
  const firstDay = (firstDayRaw + 6) % 7; // shift to 0=Mon

  const eventsForDay = (day: number) => {
    const dateStr = `${year}-${String(month + 1).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
    return events.filter((e) => e.time.startsWith(dateStr));
  };

  const deleteEvent = async (id: string) => {
    const state = await ipc.calendarDeleteEvent(id);
    setEvents(state.events);
    if (expanded?.id === id) setExpanded(null);
  };

  const prevMonth = () => {
    if (month === 0) { setYear(y => y - 1); setMonth(11); }
    else setMonth(m => m - 1);
  };
  const nextMonth = () => {
    if (month === 11) { setYear(y => y + 1); setMonth(0); }
    else setMonth(m => m + 1);
  };

  const today = new Date();
  const isToday = (day: number) =>
    day === today.getDate() && month === today.getMonth() && year === today.getFullYear();

  const selectedDayEvents = selected !== null ? eventsForDay(selected) : [];

  return (
    <div className="flex flex-col h-full overflow-hidden p-3 gap-3">
      {/* Header */}
      <div className="flex items-center justify-between gap-2 flex-shrink-0 flex-wrap">
        <div className="flex items-center gap-2 flex-wrap">
          <Button variant="ghost" size="icon-sm" onClick={prevMonth}><ChevronLeft className="h-4 w-4" /></Button>
          <span className="text-sm font-semibold w-36 text-center">{MONTHS[month]} {year}</span>
          <Button variant="ghost" size="icon-sm" onClick={nextMonth}><ChevronRight className="h-4 w-4" /></Button>
          <Button variant="ghost" size="sm" onClick={() => { setYear(today.getFullYear()); setMonth(today.getMonth()); }} className="text-xs h-7 ml-1">Today</Button>
          <div className="flex border border-border/40 rounded-full overflow-hidden p-0.5 ml-2">
            <button
              onClick={() => setView("month")}
              className={cn("rounded-full px-2.5 h-6 text-[10.5px] font-semibold transition-colors no-drag",
                view === "month" ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground")}
            >
              <CalIcon className="inline h-3 w-3 mr-1" />Month
            </button>
            <button
              onClick={() => setView("list")}
              className={cn("rounded-full px-2.5 h-6 text-[10.5px] font-semibold transition-colors no-drag",
                view === "list" ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground")}
            >
              <ListIcon className="inline h-3 w-3 mr-1" />List
            </button>
          </div>
          <button
            onClick={() => setShowInfo((v) => !v)}
            className="h-5 w-5 rounded-full border border-border/40 text-[10px] text-muted-foreground hover:text-foreground hover:border-primary transition-colors no-drag"
            title="About calendar triggers"
          >
            i
          </button>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-[11px] text-muted-foreground">{events.length} events total</span>
          <Button size="sm" onClick={() => { setAddDate(""); setShowAdd(true); }} className="h-7 text-xs">
            <Plus className="h-3.5 w-3.5" /> Schedule
          </Button>
        </div>
      </div>

      {showInfo && (
        <div className="glass-card border-primary/30 p-3 text-[11px] space-y-1.5 flex-shrink-0">
          <p className="font-semibold flex items-center gap-1.5">
            <Info className="h-3.5 w-3.5" />
            Calendar AI Triggers &amp; Referencing
          </p>
          <p className="text-muted-foreground">
            The calendar system acts as a background automation engine. When a scheduled trigger fires, the
            system spawns an AI agent with the specified trigger prompt. The agent can use files and memory
            tools to act on your workspace.
          </p>
          <ul className="text-muted-foreground list-disc list-inside space-y-0.5">
            <li>The agent will automatically run on the active model server.</li>
            <li>You can schedule tasks like: {"\"Check my project notes and extract todo items to my checklist\""}</li>
            <li>You can schedule recurring reminders or prompt commands to run while you're offline.</li>
            <li>Triggers automatically update their execution status and record model outputs for your review.</li>
          </ul>
        </div>
      )}

      <div className="flex flex-1 min-h-0 gap-3">
        {view === "month" ? (
          <>
            <div className="flex-1 flex flex-col glass-card overflow-hidden">
              <div className="grid grid-cols-7 border-b border-border/30 flex-shrink-0">
                {WEEKDAYS.map((d) => (
                  <div key={d} className="text-center py-2 text-[10px] font-bold uppercase tracking-wider text-muted-foreground">{d}</div>
                ))}
              </div>

              <div className="grid grid-cols-7 grid-rows-6 flex-1 overflow-hidden">
                {Array.from({ length: firstDay }, (_, i) => (
                  <div key={`empty-${i}`} className="border-r border-b border-border/20" />
                ))}
                {Array.from({ length: daysInMonth }, (_, i) => {
                  const day = i + 1;
                  const dayEvents = eventsForDay(day);
                  const active = selected === day;
                  return (
                    <div
                      key={day}
                      onClick={() => {
                        setSelected(active ? null : day);
                        setAddDate(`${year}-${String(month + 1).padStart(2, "0")}-${String(day).padStart(2, "0")}T09:00`);
                      }}
                      className={cn(
                        "border-r border-b border-border/20 p-1.5 cursor-pointer transition-colors overflow-hidden min-h-[80px]",
                        active ? "bg-primary/15" : "hover:bg-accent/30",
                        isToday(day) && !active ? "bg-primary/5" : ""
                      )}
                    >
                      <div className={cn(
                        "text-[11px] font-semibold w-5 h-5 flex items-center justify-center rounded-full mb-0.5",
                        isToday(day) ? "bg-primary text-primary-foreground" : "text-foreground"
                      )}>
                        {day}
                      </div>
                      <div className="space-y-0.5">
                        {dayEvents.slice(0, 3).map((ev) => (
                          <div
                            key={ev.id}
                            onClick={(e) => { e.stopPropagation(); setExpanded(ev); }}
                            className="text-[9px] truncate px-1.5 py-0.5 rounded font-semibold text-white cursor-pointer"
                            style={{ background: ev.color ?? "#6366f1" }}
                            title={`${ev.title}${ev.note ? `\n📌 ${ev.note}` : ""}`}
                          >
                            <span className="mr-0.5">
                              {ev.status === "completed" ? "✅" : ev.status === "firing" ? "⏳" : ev.status === "failed" ? "❌" : ""}
                            </span>
                            {ev.time.slice(11, 16)} {ev.title}
                          </div>
                        ))}
                        {dayEvents.length > 3 && (
                          <div className="text-[9px] text-muted-foreground pl-1 font-semibold">+{dayEvents.length - 3} more</div>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>

            <div className="w-64 flex-shrink-0 flex flex-col glass-card overflow-hidden">
              <div className="px-3 py-2 border-b border-border/40 text-[11px] font-semibold flex-shrink-0 flex items-center justify-between">
                <span>{selected ? `${MONTHS[month]} ${selected}` : "Select a day"}</span>
                {selected && (
                  <button
                    onClick={() => setSelected(null)}
                    className="text-muted-foreground hover:text-foreground no-drag"
                  >
                    <X className="h-3 w-3" />
                  </button>
                )}
              </div>
              <ScrollArea className="flex-1">
                <div className="p-2 space-y-2">
                  {selectedDayEvents.length === 0 && selected && (
                    <p className="text-xs text-muted-foreground text-center py-4">No events</p>
                  )}
                  {selectedDayEvents.map((ev) => (
                    <EventCard key={ev.id} ev={ev} onDelete={() => deleteEvent(ev.id)} onExpand={() => setExpanded(ev)} />
                  ))}
                </div>
              </ScrollArea>
              {selected && (
                <div className="p-2 border-t border-border/30 flex-shrink-0">
                  <Button
                    size="sm"
                    variant="outline"
                    className="w-full text-xs h-7"
                    onClick={() => setShowAdd(true)}
                  >
                    <Plus className="h-3 w-3" /> Add event
                  </Button>
                </div>
              )}
            </div>
          </>
        ) : (
          <ListView events={events} onDelete={deleteEvent} onExpand={setExpanded} />
        )}
      </div>

      {showAdd && (
        <AddEventDialog
          initialDate={addDate}
          onAdd={async (event) => {
            const state = await ipc.calendarAddEvent(event);
            setEvents(state.events);
            setShowAdd(false);
          }}
          onClose={() => setShowAdd(false)}
        />
      )}

      {expanded && (
        <EventDetailsDialog
          event={expanded}
          onClose={() => setExpanded(null)}
          onDelete={() => deleteEvent(expanded.id)}
        />
      )}
    </div>
  );
}

function EventCard({
  ev, onDelete, onExpand,
}: { ev: CalendarEvent; onDelete: () => void; onExpand: () => void }) {
  return (
    <div
      onClick={onExpand}
      className="rounded-md border border-border/40 bg-muted/20 p-2 space-y-1 cursor-pointer hover:border-primary/40 transition-colors group"
      style={{ borderLeft: `3px solid ${ev.color ?? "#6366f1"}` }}
    >
      <div className="flex items-start justify-between gap-1">
        <span className="text-[11px] font-semibold">
          {ev.title}
          {ev.recurrence && (
            <span className="ml-1 text-[9px] text-muted-foreground">{RECURRENCE_LABEL[ev.recurrence]}</span>
          )}
        </span>
        <button
          onClick={(e) => { e.stopPropagation(); onDelete(); }}
          className="opacity-0 group-hover:opacity-100 transition-opacity text-muted-foreground hover:text-destructive flex-shrink-0 no-drag"
        >
          <Trash2 className="h-3 w-3" />
        </button>
      </div>
      <div className="flex items-center gap-1.5 text-[9px] text-muted-foreground">
        <Clock className="h-2.5 w-2.5" />
        {ev.time.slice(11, 16)}
        <EventStatusBadge status={ev.status} />
      </div>
      {ev.note && <p className="text-[10px] text-muted-foreground line-clamp-1">📌 {ev.note}</p>}
      {ev.tags.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {ev.tags.map((t) => (
            <span key={t} className="text-[8px] bg-accent/20 text-accent-foreground px-1 rounded">
              {t}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}

function ListView({
  events, onDelete, onExpand,
}: { events: CalendarEvent[]; onDelete: (id: string) => Promise<void>; onExpand: (e: CalendarEvent) => void }) {
  const sorted = [...events].sort((a, b) => a.time.localeCompare(b.time));
  if (sorted.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center text-xs text-muted-foreground italic">
        No scheduled events.
      </div>
    );
  }
  return (
    <ScrollArea className="flex-1 glass-card p-3">
      <div className="space-y-2">
        {sorted.map((ev) => (
          <EventCard key={ev.id} ev={ev} onDelete={() => onDelete(ev.id)} onExpand={() => onExpand(ev)} />
        ))}
      </div>
    </ScrollArea>
  );
}

function EventDetailsDialog({
  event, onClose, onDelete,
}: { event: CalendarEvent; onClose: () => void; onDelete: () => void }) {
  const statusStyle: Record<string, string> = {
    pending:   "bg-violet-500 text-white",
    firing:    "bg-amber-500 text-white",
    completed: "bg-emerald-500 text-white",
    failed:    "bg-destructive text-white",
  };
  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>Event Details</DialogTitle>
        </DialogHeader>
        <div className="space-y-3">
          <div className="flex items-start justify-between gap-2">
            <div className="min-w-0">
              <p className="font-semibold text-[15px]">{event.title}</p>
              <p className="text-[11px] text-muted-foreground mt-0.5">🕒 {event.time}</p>
            </div>
            <span className={cn("text-[9px] font-bold uppercase px-2 py-0.5 rounded",
              statusStyle[event.status] ?? statusStyle.pending
            )}>
              {event.status}
            </span>
          </div>

          {event.note && (
            <div className="text-[12px] italic bg-muted/40 p-2 rounded border-l-2 border-accent">
              📌 {event.note}
            </div>
          )}

          <div className="bg-muted/40 p-2 rounded border border-border/40">
            <strong className="text-[11px]">Trigger Prompt:</strong>
            <div className="mt-1 text-[12px] font-mono whitespace-pre-wrap break-all">{event.prompt}</div>
          </div>

          {event.result && (
            <div className="bg-black/30 p-2 rounded border border-border/40 max-h-[150px] overflow-y-auto">
              <strong className="text-[11px]">Execution Output:</strong>
              <pre className="mt-1 text-[11px] font-mono whitespace-pre-wrap break-all text-indigo-300">{event.result}</pre>
            </div>
          )}

          {event.tags.length > 0 && (
            <div className="flex flex-wrap gap-1">
              {event.tags.map((t) => (
                <span key={t} className="text-[9px] bg-accent/20 text-accent-foreground px-1.5 py-0.5 rounded">
                  {t}
                </span>
              ))}
            </div>
          )}

          {event.recurrence && (
            <p className="text-[10px] text-muted-foreground">Repeats: {RECURRENCE_LABEL[event.recurrence]}</p>
          )}
        </div>
        <DialogFooter>
          <Button variant="ghost" onClick={onClose}>Close</Button>
          <Button variant="destructive" onClick={onDelete}>Delete</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function EventStatusBadge({ status }: { status: string }) {
  const map: Record<string, { label: string; variant: "default" | "success" | "warning" | "destructive" | "outline" }> = {
    pending:   { label: "Pending", variant: "outline" },
    firing:    { label: "Firing", variant: "warning" },
    completed: { label: "Done", variant: "success" },
    failed:    { label: "Failed", variant: "destructive" },
  };
  const { label, variant } = map[status] ?? { label: status, variant: "outline" };
  return <Badge variant={variant} className="text-[8px] py-0">{label}</Badge>;
}

function AddEventDialog({
  initialDate, onAdd, onClose,
}: {
  initialDate: string;
  onAdd: (event: CalendarEvent) => Promise<void>;
  onClose: () => void;
}) {
  const [title, setTitle] = useState("");
  const [time, setTime] = useState(initialDate);
  const [prompt, setPrompt] = useState("");
  const [note, setNote] = useState("");
  const [color, setColor] = useState("#6366f1");
  const [recurrence, setRecurrence] = useState("");
  const [tagsInput, setTagsInput] = useState("");
  const [tags, setTags] = useState<string[]>([]);
  const [saving, setSaving] = useState(false);

  const addTag = () => {
    const t = tagsInput.trim();
    if (t && !tags.includes(t)) setTags([...tags, t]);
    setTagsInput("");
  };

  const submit = async () => {
    if (!title.trim() || !time || !prompt.trim()) return;
    setSaving(true);
    const event: CalendarEvent = {
      id: `e${Date.now()}`,
      title: title.trim(),
      time,
      prompt: prompt.trim(),
      note: note.trim() || null,
      color,
      status: "pending",
      result: null,
      tags,
      recurrence: recurrence || null,
    };
    await onAdd(event);
    setSaving(false);
  };

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogHeader><DialogTitle>Schedule AI Prompt</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div className="space-y-1">
            <Label className="text-xs">Title</Label>
            <Input value={title} onChange={(e) => setTitle(e.target.value)} placeholder="Daily standup summary" />
          </div>
          <div className="space-y-1">
            <Label className="text-xs">Date & time</Label>
            <Input type="datetime-local" value={time} onChange={(e) => setTime(e.target.value)} />
          </div>
          <div className="space-y-1">
            <Label className="text-xs">AI prompt (runs at scheduled time)</Label>
            <textarea
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              rows={3}
              placeholder="Summarise today's commits and open PRs…"
              className="flex w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring resize-none no-drag"
            />
          </div>
          <div className="space-y-1">
            <Label className="text-xs">Note (optional)</Label>
            <Input value={note} onChange={(e) => setNote(e.target.value)} placeholder="Context or reminder" />
          </div>
          <div className="flex items-center gap-2 flex-wrap">
            <Label className="text-xs">Color</Label>
            <div className="flex gap-1.5">
              {COLORS.map((c) => (
                <button
                  key={c}
                  onClick={() => setColor(c)}
                  className={cn("h-5 w-5 rounded-full border-2 transition-all no-drag",
                    color === c ? "border-foreground scale-110" : "border-transparent hover:scale-105"
                  )}
                  style={{ background: c }}
                  title={c}
                />
              ))}
            </div>
          </div>
          <div className="space-y-1">
            <Label className="text-xs">Recurrence</Label>
            <select
              value={recurrence}
              onChange={(e) => setRecurrence(e.target.value)}
              className="w-full h-8 text-xs bg-transparent border border-input rounded-md px-2 no-drag"
            >
              <option value="">One-time</option>
              <option value="daily">🔁 Daily</option>
              <option value="weekly">📅 Weekly</option>
              <option value="monthly">🗓 Monthly</option>
            </select>
          </div>
          <div className="space-y-1">
            <Label className="text-xs">Tags</Label>
            <div className="flex flex-wrap gap-1 mb-1">
              {tags.map((t, i) => (
                <span key={t} className="inline-flex items-center gap-1 text-[10px] bg-accent/20 text-accent-foreground rounded px-1.5 py-0.5">
                  {t}
                  <button
                    onClick={() => setTags(tags.filter((_, j) => j !== i))}
                    className="text-accent-foreground/60 hover:text-foreground no-drag"
                  >
                    ×
                  </button>
                </span>
              ))}
            </div>
            <div className="flex gap-1">
              <Input
                value={tagsInput}
                onChange={(e) => setTagsInput(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") { e.preventDefault(); addTag(); }
                }}
                placeholder="Add tag, press Enter…"
                className="h-7 text-[11px]"
              />
              <Button size="sm" variant="outline" onClick={addTag} className="h-7 text-[10px] no-drag">Add</Button>
            </div>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>Cancel</Button>
          <Button onClick={submit} disabled={saving || !title.trim() || !prompt.trim() || !time}>
            {saving ? "Scheduling…" : "Schedule"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
