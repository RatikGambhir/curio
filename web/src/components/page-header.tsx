import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

export function PageHeader({
  children,
  className,
  ...props
}: ComponentProps<"header">) {
  return (
    <header
      className={cn(
        "flex h-[2.75rem] shrink-0 items-center gap-3 border-b-[0.5px] border-border bg-card px-[1.25rem] md:h-[calc(var(--app-header-height)-0.5px)]",
        className,
      )}
      {...props}
    >
      {children}
    </header>
  );
}
