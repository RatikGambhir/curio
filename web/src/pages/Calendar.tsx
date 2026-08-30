import { useCallback, useEffect, useMemo, useState } from "react";

import type { CalendarView } from "@/api/calendar";
import { AppSidebar } from "@/components/app-sidebar";
import {
  calendarPermissions,
  labelOptions,
  priorityOptions,
  statusColors,
  statusOptions,
} from "@/components/calendar/calendar.config";
import type {
  CalendarPriority,
  CalendarStatus,
  CurioCalendarEvent,
} from "@/components/calendar/calendar.types";
import type {
  TaskItem,
  TaskItemAddedEvent,
} from "@/components/event-calendar";
import { EventCalendar } from "@/components/event-calendar";
import { calendarEditing } from "@/components/event-calendar/features/editing";
import { PageHeader } from "@/components/page-header";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import {
  useCalendarEvents,
  useCreateCalendarEvent,
  type VisibleCalendarRange,
} from "@/hooks/useCalendarEvents";

const DATE_ONLY = /^\d{4}-\d{2}-\d{2}$/;

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
const isCalendarStatus = (value: string): value is CalendarStatus =>
  statusOptions.some((option) => option.value === value);
const isCalendarPriority = (
  value: string | undefined,
): value is CalendarPriority =>
  value !== undefined &&
  priorityOptions.some((option) => option.value === value);

const Calendar = () => {
  // Stable identity: an inline `new Date()` would take a fresh identity every
  // render and drive onRangeChange into a loop.
  const [now] = useState(() => new Date());
  const [visibleRange, setVisibleRange] =
    useState<VisibleCalendarRange | null>(null);
  const calendarQuery = useCalendarEvents(visibleRange);
  const { mutate: createEvent, error: createError } =
    useCreateCalendarEvent(visibleRange);
  const [events, setEvents] = useState<CurioCalendarEvent[]>([]);
  const [selectedEventId, setSelectedEventId] = useState<string | null>(null);

  const loadedEvents = calendarQuery.data?.events;
  useEffect(() => {
    if (!loadedEvents) {
      return;
    }
    setEvents(loadedEvents);
    setSelectedEventId((currentId) =>
      findEvent(loadedEvents, currentId) ? currentId : null,
    );
  }, [loadedEvents]);

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

  const handleRangeChange = useCallback(
    (range: { view: CalendarView; start: Date; end: Date }) => {
      setVisibleRange({
        view: range.view,
        start: range.start.toISOString(),
        end: range.end.toISOString(),
      });
    },
    [],
  );

  const handleItemAdded = useCallback(
    ({ item }: TaskItemAddedEvent) => {
      const startDate = item.startAt ?? item.setAt;
      createEvent({
        id: item.id,
        title: item.name,
        description: item.description ?? null,
        status: isCalendarStatus(item.status) ? item.status : null,
        priority: isCalendarPriority(item.priority) ? item.priority : null,
        allDay: DATE_ONLY.test(startDate),
        startDate,
        endDate: item.expireAt ?? null,
      });
    },
    [createEvent],
  );

  const serviceMessage = useMemo(() => {
    if (createError instanceof Error) {
      return createError.message;
    }
    if (calendarQuery.error instanceof Error) {
      return calendarQuery.error.message;
    }
    if (calendarQuery.isPending && visibleRange) {
      return "Loading events…";
    }
    return null;
  }, [calendarQuery.error, calendarQuery.isPending, createError, visibleRange]);

  const selectionLabel = useMemo(() => {
    if (serviceMessage) {
      return serviceMessage;
    }
    if (!selectedEvent) {
      return "Select an event to see it here";
    }
    return `${selectedEvent.name} · ${selectedEvent.status}`;
  }, [selectedEvent, serviceMessage]);

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
              onItemAdded={handleItemAdded}
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
