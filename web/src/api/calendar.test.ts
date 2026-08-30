import { afterEach, describe, expect, it, vi } from "vitest"

import { createCalendarEvent, listCalendarEvents } from "@/api/calendar"

describe("calendar api", () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it("sends the exact bounds computed by the visible calendar view", async () => {
    const fetchMock = vi.fn<typeof fetch>(
      async () => new Response(JSON.stringify({ events: [] })),
    )
    vi.stubGlobal("fetch", fetchMock)

    await listCalendarEvents({
      userId: "user-1",
      view: "month",
      start: "2026-05-31T07:00:00.000Z",
      end: "2026-07-01T07:00:00.000Z",
    })

    const requested = new URL(String(fetchMock.mock.calls[0][0]))
    expect(requested.pathname).toBe("/v1/calendar/events")
    expect(requested.searchParams.get("userId")).toBe("user-1")
    expect(requested.searchParams.get("view")).toBe("month")
    expect(requested.searchParams.get("start")).toBe(
      "2026-05-31T07:00:00.000Z",
    )
    expect(requested.searchParams.get("end")).toBe(
      "2026-07-01T07:00:00.000Z",
    )
  })

  it("returns date-only all-day strings untouched", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(
        async () =>
          new Response(
            JSON.stringify({
              events: [
                {
                  id: "event-1",
                  userId: "user-1",
                  title: "Offsite",
                  description: null,
                  status: "scheduled",
                  priority: "medium",
                  allDay: true,
                  startDate: "2026-06-22",
                  endDate: "2026-06-25",
                  createdAt: "2026-06-01 00:00:00",
                  updatedAt: "2026-06-01 00:00:00",
                },
              ],
            }),
          ),
      ),
    )

    const { events } = await listCalendarEvents({
      userId: "user-1",
      view: "month",
      start: "2026-05-31T07:00:00.000Z",
      end: "2026-07-01T07:00:00.000Z",
    })

    expect(events[0].startDate).toBe("2026-06-22")
    expect(events[0].endDate).toBe("2026-06-25")
  })

  it("posts a milestone without adding an end date", async () => {
    const fetchMock = vi.fn<typeof fetch>(
      async () =>
        new Response(
          JSON.stringify({
            id: "event-1",
            userId: "user-1",
            title: "Launch",
            description: null,
            status: "scheduled",
            priority: null,
            allDay: false,
            startDate: "2026-06-22T14:00:00.000Z",
            endDate: null,
            createdAt: "2026-06-01 00:00:00",
            updatedAt: "2026-06-01 00:00:00",
          }),
          { status: 201 },
        ),
    )
    vi.stubGlobal("fetch", fetchMock)

    const event = await createCalendarEvent(
      {
        userId: "user-1",
        title: "Launch",
        status: "scheduled",
        allDay: false,
        startDate: "2026-06-22T14:00:00.000Z",
      },
      { bearerToken: "user-1" },
    )

    const [, init] = fetchMock.mock.calls[0]
    expect(JSON.parse(String(init?.body))).toMatchObject({
      userId: "user-1",
      allDay: false,
      startDate: "2026-06-22T14:00:00.000Z",
    })
    expect(event.endDate).toBeNull()
  })
})
