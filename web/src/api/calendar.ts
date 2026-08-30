import { api, type ApiRequestOptions } from "@/api/client"
import type {
  CalendarPriority,
  CalendarStatus,
} from "@/components/calendar/calendar.types"

export type CalendarView = "day" | "week" | "month" | "agenda"

/** A stored event exactly as the service returns it. */
export type CalendarEventRecord = {
  id: string
  userId: string
  title: string
  description: string | null
  status: CalendarStatus | null
  priority: CalendarPriority | null
  allDay: boolean
  startDate: string
  endDate: string | null
  createdAt: string
  updatedAt: string
}

export type CalendarRange = {
  userId: string
  view: CalendarView
  /** Bounds already converted to UTC from the caller's local calendar view. */
  start: string
  end: string
}

export type CreateCalendarEventInput = {
  id?: string
  userId: string
  title: string
  description?: string | null
  status?: CalendarStatus | null
  priority?: CalendarPriority | null
  allDay: boolean
  startDate: string
  endDate?: string | null
}

export function listCalendarEvents(
  range: CalendarRange,
  options?: ApiRequestOptions,
): Promise<{ events: CalendarEventRecord[] }> {
  return api.get("/v1/calendar/events", { ...range }, options)
}

export function createCalendarEvent(
  input: CreateCalendarEventInput,
  options?: ApiRequestOptions,
): Promise<CalendarEventRecord> {
  return api.post("/v1/calendar/events", input, options)
}
