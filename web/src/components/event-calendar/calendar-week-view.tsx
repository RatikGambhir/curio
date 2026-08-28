import { useMemo } from "react"
import { addDays } from "date-fns"

import { CalendarTimeGrid } from "./calendar-time-grid"
import { useCalendar } from "./calendar-context"

export function CalendarWeekView() {
  const { range } = useCalendar()
  const days = useMemo(
    () => Array.from({ length: 7 }, (_, index) => addDays(range.start, index)),
    [range.start],
  )

  return <CalendarTimeGrid days={days} />
}
