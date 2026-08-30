import { useCallback, useMemo, useState } from "react";

import { AppSidebar } from "@/components/app-sidebar";
import {
  calendarPermissions,
  labelOptions,
  priorityOptions,
  statusColors,
  statusOptions,
} from "@/components/calendar/calendar.config";
import { buildMockCalendarEvents } from "@/components/calendar/calendar.mock-data";
import type { CurioCalendarEvent } from "@/components/calendar/calendar.types";
import type { TaskItem } from "@/components/event-calendar";
import { EventCalendar } from "@/components/event-calendar";
import { calendarEditing } from "@/components/event-calendar/features/editing";
import { PageHeader } from "@/components/page-header";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";

function findEvent(
  items: readonly TaskItem[],
  eventId: string | null,
): TaskItem | undefined {
  if (!eventId) {
    return undefined;
  }

  for (const item of items) {
    if (item.id === eventId) {
      return item;
    }
    const child = findEvent(item.children ?? [], eventId);
    if (child) {
      return child;
    }
  }

  return undefined;
}

const isHighPriority = (item: TaskItem) => item.priority === "high";

const Calendar = () => {
  // Stable identity: an inline `new Date()` would take a fresh identity every
  // render and drive onRangeChange into a loop.
  const [now] = useState(() => new Date());
  const [events, setEvents] = useState<CurioCalendarEvent[]>(() =>
    buildMockCalendarEvents(now),
  );
  const [selectedEventId, setSelectedEventId] = useState<string | null>(null);

  const selectedEvent = useMemo(
    () => findEvent(events, selectedEventId),
    [events, selectedEventId],
  );

  const handleEventsChange = useCallback((nextEvents: TaskItem[]) => {
    setEvents(nextEvents);
    setSelectedEventId((currentId) =>
      findEvent(nextEvents, currentId) ? currentId : null,
    );
  }, []);

  const handleTaskClick = useCallback((task: TaskItem) => {
    setSelectedEventId(task.id);
  }, []);

  const handleRangeChange = useCallback(() => {
    // Phase 1 renders every event from memory. When `GET /v1/events?start=&end=`
    // exists this is where the visible range drives the query — through
    // src/api/, never a network call from page code.
  }, []);

  const renderTooltip = useCallback(
    (task: TaskItem) => (
      <div className="flex flex-col gap-1">
        <span className="font-medium">{task.name}</span>
        {task.description ? (
          <span className="text-muted-foreground">{task.description}</span>
        ) : null}
        <span className="text-xs text-muted-foreground">
          {task.status ?? "unscheduled"}
          {task.priority ? ` · ${task.priority} priority` : ""}
        </span>
      </div>
    ),
    [],
  );

  const selectionLabel = useMemo(() => {
    if (!selectedEvent) {
      return "Select an event to see it here";
    }
    return `${selectedEvent.name} · ${selectedEvent.status}`;
  }, [selectedEvent]);

  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset className="bg-background">
        <div className="flex h-screen w-full flex-col bg-background">
          <PageHeader />
          <div className="flex shrink-0 items-center justify-between gap-3 px-4 pt-3 pb-2">
            <h1 className="text-lg font-semibold text-foreground">Calendar</h1>
            <span className="truncate text-xs text-muted-foreground">
              {selectionLabel}
            </span>
          </div>
          <main className="min-h-0 flex-1 px-4 pb-4">
            <EventCalendar
              data={events}
              onChange={handleEventsChange}
              statusOptions={statusOptions}
              priorityOptions={priorityOptions}
              labelOptions={labelOptions}
              statusColors={statusColors}
              flagPriority={isHighPriority}
              defaultView="month"
              now={now}
              showMiniNav
              editable
              editing={calendarEditing}
              permissions={calendarPermissions}
              selectedId={selectedEventId}
              onSelect={setSelectedEventId}
              onTaskClick={handleTaskClick}
              onRangeChange={handleRangeChange}
              renderTooltip={renderTooltip}
              className="h-full"
            />
          </main>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
};

export default Calendar;
