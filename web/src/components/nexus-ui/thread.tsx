"use client";

import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { ArrowDown } from "lucide-react";
import { StickToBottom, useStickToBottomContext } from "use-stick-to-bottom";

import { cn } from "@/lib/utils";

type ThreadProps = React.ComponentProps<typeof StickToBottom>;

function Thread({
  className,
  resize = "smooth",
  initial = "smooth",
  ...props
}: ThreadProps) {
  return (
    <StickToBottom
      data-slot="thread"
      className={cn("relative w-full h-full", className)}
      resize={resize}
      initial={initial}
      {...props}
    />
  );
}

type ThreadContentProps = React.ComponentProps<typeof StickToBottom.Content>;

function ThreadContent({ className, ...props }: ThreadContentProps) {
  return (
    <StickToBottom.Content
      data-slot="thread-content"
      className={cn(
        "flex w-full flex-col gap-6 p-6",
        className,
      )}
      {...props}
    />
  );
}

type ThreadScrollToBottomProps = React.ComponentProps<"button"> & {
  asChild?: boolean;
};

function ThreadScrollToBottom({
  asChild = false,
  className,
  children,
  onClick,
  ...props
}: ThreadScrollToBottomProps) {
  const { isAtBottom, scrollToBottom } = useStickToBottomContext();

  if (isAtBottom) {
    return null;
  }

  const Comp = asChild ? Slot : "button";

  return (
    <Comp
      data-slot="thread-scroll-to-bottom"
      type={asChild ? undefined : "button"}
      className={cn(
        !asChild &&
          "focus-ring absolute bottom-4 left-[50%] flex size-8 translate-x-[-50%] cursor-pointer items-center justify-center rounded-full border border-border bg-card text-muted-foreground shadow-md transition-colors hover:text-foreground",
        className,
      )}
      onClick={(event) => {
        scrollToBottom();
        onClick?.(event);
      }}
      {...props}
    >
      {children ?? (
        <>
          <ArrowDown className="size-4" aria-hidden="true" />
          <span className="sr-only">Scroll to latest message</span>
        </>
      )}
    </Comp>
  );
}

export { Thread, ThreadContent, ThreadScrollToBottom };
