import { api, type ApiRequestOptions } from "@/api/client"

/**
 * The views the calendar can request. The view does not decide the range — the
 * client sends explicit bounds — but the service uses it to cap how wide a
 * single request may be.
 */
export type CalendarView = "day" | "week" | "month" | "agenda"

export const CALENDAR_EVENT_STATUSES = [
  "scheduled",
  "confirmed",
  "tentative",
  "cancelled",
] as const

export const CALENDAR_EVENT_PRIORITIES = ["low", "medium", "high"] as const

export type CalendarEventStatus = (typeof CALENDAR_EVENT_STATUSES)[number]
export type CalendarEventPriority = (typeof CALENDAR_EVENT_PRIORITIES)[number]

/**
 * A stored event exactly as the service returns it.
 *
 * `startDate` and `endDate` are format-bearing: a bare `YYYY-MM-DD` is an
 * all-day event, a full timestamp is a timed one, and a timestamp with no end
 * is a milestone. Nothing in this path may normalise them — a well-meaning
 * `new Date(value).toISOString()` turns every all-day event into a midnight
 * block, and the failure is silent.
 */
export type CurioCalendarEvent = {
  id: string
  userId: string
  title: string
  description: string | null
  status: CalendarEventStatus | null
  priority: CalendarEventPriority | null
  allDay: boolean
  startDate: string
  endDate: string | null
  createdAt: string
  updatedAt: string
}

export type CalendarRange = {
  userId: string
  view: CalendarView
  /** The view's bounds, already converted to UTC from the caller's local view. */
  start: string
  end: string
}

export type CreateCalendarEventInput = {
  /** Client-generated, which is what makes a retried create idempotent. */
  id?: string
  userId: string
  title: string
  description?: string | null
  status?: CalendarEventStatus | null
  priority?: CalendarEventPriority | null
  allDay: boolean
  startDate: string
  endDate?: string | null
}

export function listCalendarEvents(
  range: CalendarRange,
  options?: ApiRequestOptions,
): Promise<{ events: CurioCalendarEvent[] }> {
  return api.get("/v1/calendar/events", { ...range }, options)
}

export function createCalendarEvent(
  input: CreateCalendarEventInput,
  options?: ApiRequestOptions,
): Promise<CurioCalendarEvent> {
  return api.post<CurioCalendarEvent>("/v1/calendar/events", input, options)
}
