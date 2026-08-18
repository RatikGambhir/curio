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
        "flex h-16 shrink-0 items-center gap-3 border-b border-border bg-background px-4",
        className,
      )}
      {...props}
    >
      {children}
    </header>
  );
}
