import { useState, useEffect } from "react";
import { ChevronLeft, ChevronRight, Plus, Trash2, Clock } from "lucide-react";
import { ipc, type CalendarEvent } from "@/lib/ipc";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";

const WEEKDAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS = ["January","February","March","April","May","June","July","August","September","October","November","December"];

export function CalendarTab() {
  const [events, setEvents] = useState<CalendarEvent[]>([]);
  const [year, setYear] = useState(new Date().getFullYear());
  const [month, setMonth] = useState(new Date().getMonth());
  const [selected, setSelected] = useState<number | null>(null);
  const [showAdd, setShowAdd] = useState(false);
  const [addDate, setAddDate] = useState("");

  useEffect(() => {
    ipc.calendarLoad().then((s) => setEvents(s.events));
  }, []);

  const daysInMonth = new Date(year, month + 1, 0).getDate();
  const firstDay = new Date(year, month, 1).getDay();

  const eventsForDay = (day: number) => {
    const dateStr = `${year}-${String(month + 1).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
    return events.filter((e) => e.time.startsWith(dateStr));
  };

  const deleteEvent = async (id: string) => {
    const state = await ipc.calendarDeleteEvent(id);
    setEvents(state.events);
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
  const todayStr = `${today.getFullYear()}-${today.getMonth()}-${today.getDate()}`;
  const isToday = (day: number) =>
    day === today.getDate() && month === today.getMonth() && year === today.getFullYear();

  const selectedDayEvents = selected !== null ? eventsForDay(selected) : [];

  return (
    <div className="flex flex-col h-full overflow-hidden p-3 gap-3">
      {/* Header */}
      <div className="flex items-center justify-between flex-shrink-0">
        <div className="flex items-center gap-2">
          <Button variant="ghost" size="icon-sm" onClick={prevMonth}><ChevronLeft className="h-4 w-4" /></Button>
          <span className="text-sm font-semibold w-36 text-center">{MONTHS[month]} {year}</span>
          <Button variant="ghost" size="icon-sm" onClick={nextMonth}><ChevronRight className="h-4 w-4" /></Button>
          <Button variant="ghost" size="sm" onClick={() => { setYear(today.getFullYear()); setMonth(today.getMonth()); }} className="text-xs h-7 ml-1">Today</Button>
        </div>
        <Button size="sm" onClick={() => { setAddDate(""); setShowAdd(true); }} className="h-7 text-xs">
          <Plus className="h-3.5 w-3.5" /> Schedule
        </Button>
      </div>

      <div className="flex flex-1 min-h-0 gap-3">
        {/* Calendar grid */}
        <div className="flex-1 flex flex-col glass-card overflow-hidden">
          {/* Weekday headers */}
          <div className="grid grid-cols-7 border-b border-border/30 flex-shrink-0">
            {WEEKDAYS.map((d) => (
              <div key={d} className="text-center py-2 text-[10px] font-medium text-muted-foreground">{d}</div>
            ))}
          </div>

          {/* Day cells */}
          <div className="grid grid-cols-7 flex-1 overflow-hidden">
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
                  onClick={() => setSelected(active ? null : day)}
                  className={cn(
                    "border-r border-b border-border/20 p-1 cursor-pointer transition-colors overflow-hidden",
                    active ? "bg-primary/15" : "hover:bg-accent/50",
                    isToday(day) && !active ? "bg-primary/5" : ""
                  )}
                >
                  <div className={cn(
                    "text-[11px] font-medium w-5 h-5 flex items-center justify-center rounded-full mb-0.5",
                    isToday(day) ? "bg-primary text-primary-foreground" : "text-foreground"
                  )}>
                    {day}
                  </div>
                  <div className="space-y-0.5">
                    {dayEvents.slice(0, 2).map((ev) => (
                      <div
                        key={ev.id}
                        className="text-[9px] truncate px-1 py-0.5 rounded"
                        style={{ background: ev.color ?? "hsl(var(--primary) / 0.15)", color: ev.color ?? "hsl(var(--primary))" }}
                      >
                        {ev.title}
                      </div>
                    ))}
                    {dayEvents.length > 2 && (
                      <div className="text-[9px] text-muted-foreground pl-1">+{dayEvents.length - 2}</div>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Day detail */}
        <div className="w-56 flex-shrink-0 flex flex-col glass-card overflow-hidden">
          <div className="px-3 py-2 border-b border-border/40 text-[11px] font-semibold flex-shrink-0">
            {selected ? `${MONTHS[month]} ${selected}` : "Select a day"}
          </div>
          <ScrollArea className="flex-1">
            <div className="p-2 space-y-2">
              {selectedDayEvents.length === 0 && selected && (
                <p className="text-xs text-muted-foreground text-center py-4">No events</p>
              )}
              {selectedDayEvents.map((ev) => (
                <div key={ev.id} className="glass-card p-2 space-y-1 group">
                  <div className="flex items-start justify-between gap-1">
                    <span className="text-[11px] font-medium">{ev.title}</span>
                    <button
                      onClick={() => deleteEvent(ev.id)}
                      className="opacity-0 group-hover:opacity-100 transition-opacity text-muted-foreground hover:text-destructive no-drag"
                    >
                      <Trash2 className="h-3 w-3" />
                    </button>
                  </div>
                  <div className="flex items-center gap-1 text-[9px] text-muted-foreground">
                    <Clock className="h-2.5 w-2.5" />
                    {ev.time.slice(11, 16)}
                  </div>
                  <EventStatusBadge status={ev.status} />
                  {ev.note && <p className="text-[10px] text-muted-foreground">{ev.note}</p>}
                </div>
              ))}
            </div>
          </ScrollArea>
          {selected && (
            <div className="p-2 border-t border-border/30 flex-shrink-0">
              <Button
                size="sm"
                variant="outline"
                className="w-full text-xs h-7"
                onClick={() => {
                  setAddDate(`${year}-${String(month + 1).padStart(2, "0")}-${String(selected).padStart(2, "0")}T09:00`);
                  setShowAdd(true);
                }}
              >
                <Plus className="h-3 w-3" /> Add event
              </Button>
            </div>
          )}
        </div>
      </div>

      {/* Add event dialog */}
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
    </div>
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
  return <Badge variant={variant} className="text-[9px] py-0">{label}</Badge>;
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
  const [saving, setSaving] = useState(false);

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
      tags: [],
      recurrence: null,
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
          <div className="flex items-center gap-2">
            <Label className="text-xs">Color</Label>
            <input type="color" value={color} onChange={(e) => setColor(e.target.value)} className="h-7 w-10 cursor-pointer rounded border border-input p-0.5 no-drag" />
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
