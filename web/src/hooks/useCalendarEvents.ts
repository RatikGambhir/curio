import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"

import {
  createCalendarEvent,
  listCalendarEvents,
  type CalendarEventRecord,
  type CalendarRange,
  type CreateCalendarEventInput,
} from "@/api/calendar"
import type { CurioCalendarEvent } from "@/components/calendar/calendar.types"
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser"

export type VisibleCalendarRange = Omit<CalendarRange, "userId">

export function calendarEventsKey(
  userId: string | undefined,
  range: VisibleCalendarRange | null,
) {
  return [
    "calendar",
    userId ?? null,
    range?.view ?? null,
    range?.start ?? null,
    range?.end ?? null,
  ] as const
}

export function calendarEventsKeyPrefix(userId: string | undefined) {
  return ["calendar", userId ?? null] as const
}

/** Preserve the date strings: their format controls all-day/timed rendering. */
export function calendarRecordToEvent(
  record: CalendarEventRecord,
): CurioCalendarEvent {
  return {
    id: record.id,
    name: record.title,
    description: record.description ?? undefined,
    status: record.status ?? "scheduled",
    active: true,
    priority: record.priority ?? undefined,
    setAt: record.startDate,
    expireAt: record.endDate ?? undefined,
    source: "user",
  }
}

export function useCalendarEvents(range: VisibleCalendarRange | null) {
  const { user } = useAuthenticatedUser()
  const userId = user?.id

  return useQuery({
    queryKey: calendarEventsKey(userId, range),
    queryFn: ({ signal }) => {
      if (!userId || !range) {
        throw new Error("A calendar range needs a signed-in user.")
      }
      return listCalendarEvents(
        { userId, ...range },
        { bearerToken: userId, signal },
      )
    },
    select: (response) => ({
      events: response.events.map(calendarRecordToEvent),
    }),
    enabled: Boolean(userId && range),
  })
}

export function useCreateCalendarEvent() {
  const { user } = useAuthenticatedUser()
  const queryClient = useQueryClient()
  const userId = user?.id

  return useMutation({
    mutationFn: (input: Omit<CreateCalendarEventInput, "userId">) => {
      if (!userId) {
        throw new Error("An event needs a signed-in user to own it.")
      }
      return createCalendarEvent({ ...input, userId }, { bearerToken: userId })
    },
    // Always reconcile every cached range with server truth. A successful
    // create can overlap more than the range it was composed in, while a
    // rejected POST must leave the controlled calendar unchanged.
    onSettled: () =>
      queryClient.invalidateQueries({
        queryKey: calendarEventsKeyPrefix(userId),
      }),
  })
}
