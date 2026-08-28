import { useCallback, useMemo, useState } from "react"

import { AppSidebar } from "@/components/app-sidebar"
import {
  calendarPermissions,
  labelOptions,
  priorityOptions,
  statusColors,
  statusOptions,
} from "@/components/calendar/calendar.config"
import { buildMockCalendarEvents } from "@/components/calendar/calendar.mock-data"
import type { CurioCalendarEvent } from "@/components/calendar/calendar.types"
import { EventCalendar, calendarEditing } from "@/components/event-calendar"
import { PageHeader } from "@/components/page-header"
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar"

const Calendar = () => {
  // Stable identity: an inline `new Date()` would take a fresh identity every
  // render and drive onRangeChange into a loop.
  const [now] = useState(() => new Date())
  const [events, setEvents] = useState<CurioCalendarEvent[]>(() =>
    buildMockCalendarEvents(now),
  )
  const [selectedEvent, setSelectedEvent] = useState<CurioCalendarEvent | null>(
    null,
  )

  const handleRangeChange = useCallback(() => {
    // Phase 1 renders every event from memory. When `GET /v1/events?start=&end=`
    // exists this is where the visible range drives the query — through
    // src/api/, never a network call from page code.
  }, [])

  const renderTooltip = useCallback(
    (task: CurioCalendarEvent) => (
      <div className="flex flex-col gap-1">
        <span className="font-medium">{task.title}</span>
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
  )

  const selectionLabel = useMemo(() => {
    if (!selectedEvent) {
      return "Select an event to see it here"
    }
    return `${selectedEvent.title} · ${selectedEvent.status ?? "unscheduled"}`
  }, [selectedEvent])

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
              onChange={setEvents}
              statusOptions={statusOptions}
              priorityOptions={priorityOptions}
              labelOptions={labelOptions}
              statusColors={statusColors}
              defaultView="month"
              now={now}
              showMiniNav
              editable
              editing={calendarEditing}
              permissions={calendarPermissions}
              onTaskClick={setSelectedEvent}
              onRangeChange={handleRangeChange}
              renderTooltip={renderTooltip}
              className="h-full"
            />
          </main>
        </div>
      </SidebarInset>
    </SidebarProvider>
  )
}

export default Calendar
