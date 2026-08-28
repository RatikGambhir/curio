import { useMemo } from "react"

import { CalendarTimeGrid } from "./calendar-time-grid"
import { useCalendar } from "./calendar-context"

export function CalendarDayView() {
  const { range } = useCalendar()
  const days = useMemo(() => [range.start], [range.start])

  return <CalendarTimeGrid days={days} />
}
