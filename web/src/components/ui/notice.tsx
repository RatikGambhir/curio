import type { ReactNode } from "react"
import { AlertTriangle, Info } from "lucide-react"

import { cn } from "@/lib/utils"

/* An inline message that belongs to a region of the page — a failed load, a
   limitation worth stating — rather than a toast that disappears. Errors are
   announced; informational notes are not. */
export function Notice({
  tone = "info",
  title,
  children,
  action,
  className,
}: {
  tone?: "info" | "error"
  title: ReactNode
  children?: ReactNode
  action?: ReactNode
  className?: string
}) {
  const Icon = tone === "error" ? AlertTriangle : Info

  return (
    <div
      role={tone === "error" ? "alert" : "note"}
      className={cn(
        "flex items-start gap-3 rounded-md border px-3.5 py-3 text-sm",
        tone === "error"
          ? "border-destructive/30 bg-destructive/6 text-foreground"
          : "border-border bg-secondary/60 text-foreground",
        className,
      )}
    >
      <Icon
        className={cn(
          "mt-0.5 size-4 shrink-0",
          tone === "error" ? "text-destructive" : "text-muted-foreground",
        )}
        aria-hidden="true"
      />
      <div className="min-w-0 flex-1">
        <p className="font-medium">{title}</p>
        {children ? (
          <div className="mt-0.5 text-muted-foreground">{children}</div>
        ) : null}
      </div>
      {action ? <div className="shrink-0 self-center">{action}</div> : null}
    </div>
  )
}
