import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"

import {
  createCalendarEvent,
  listCalendarEvents,
  type CalendarRange,
  type CreateCalendarEventInput,
} from "@/api/calendar"
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser"

/** The bounds the grid is currently drawing, minus the owner it is drawn for. */
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

/**
 * Reads the events overlapping the range the calendar is currently showing.
 *
 * The range comes from whatever the grid computed for rendering rather than
 * from a view name the server re-derives, so the fetch and the display cannot
 * disagree about where a month or a week begins.
 *
 * As in `useSaveUser`, the mock auth session's user id doubles as both the
 * placeholder bearer token and the scoping key, so the two move together when
 * real auth lands.
 */
export function useCalendarEvents(range: VisibleCalendarRange | null) {
  const { user } = useAuthenticatedUser()
  const userId = user?.id

  return useQuery({
    queryKey: calendarEventsKey(userId, range),
    queryFn: () => {
      if (!userId || !range) {
        throw new Error("A calendar range needs a signed-in user.")
      }
      return listCalendarEvents({ userId, ...range }, { bearerToken: userId })
    },
    enabled: Boolean(userId && range),
  })
}

/**
 * Creates an event, then invalidates the visible range so the grid picks it up.
 */
export function useCreateCalendarEvent(range: VisibleCalendarRange | null) {
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
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: calendarEventsKey(userId, range),
      })
    },
  })
}
