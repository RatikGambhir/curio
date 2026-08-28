import { Skeleton } from "@/components/ui/skeleton"

export function CalendarSkeleton() {
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3 p-3" aria-hidden>
      <div className="flex items-center justify-between">
        <Skeleton className="h-8 w-44" />
        <Skeleton className="h-8 w-64" />
      </div>
      <div className="grid min-h-0 flex-1 grid-cols-7 gap-px">
        {Array.from({ length: 35 }, (_, index) => (
          <Skeleton key={index} className="min-h-20 rounded-sm" />
        ))}
      </div>
    </div>
  )
}
