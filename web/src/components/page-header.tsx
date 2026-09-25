import type { ReactNode } from "react";

import { SidebarTrigger } from "@/components/ui/sidebar";
import { cn } from "@/lib/utils";

/* The toolbar every page inside the app shell opens with. It is the same height
   as the spine header and the context-pane header, so their rules meet in one
   line. The title is the page's h1 unless the page sets its own masthead, in
   which case it is demoted to a label and the masthead carries the heading. */
export function PageHeader({
  title,
  meta,
  leading,
  actions,
  titleIsHeading = true,
  className,
}: {
  title: ReactNode;
  /** Short index mark beside the title: a date, a count, a status. */
  meta?: ReactNode;
  /** Controls before the title, such as a context-pane toggle. */
  leading?: ReactNode;
  /** Controls at the trailing edge. */
  actions?: ReactNode;
  titleIsHeading?: boolean;
  className?: string;
}) {
  const Title = titleIsHeading ? "h1" : "p";

  return (
    <header
      className={cn(
        "flex h-(--app-header-height) shrink-0 items-center gap-2 border-b border-border px-3 sm:px-5",
        className,
      )}
    >
      <SidebarTrigger className="-ml-1 text-muted-foreground md:hidden" />
      {leading}
      <div className="flex min-w-0 flex-1 items-baseline gap-3">
        <Title className="truncate font-display text-[1.25rem] leading-none tracking-[-0.01em] text-foreground">
          {title}
        </Title>
        {meta ? (
          <span className="eyebrow hidden truncate text-muted-foreground sm:inline">
            {meta}
          </span>
        ) : null}
      </div>
      {actions ? (
        <div className="flex shrink-0 items-center gap-2">{actions}</div>
      ) : null}
    </header>
  );
}
