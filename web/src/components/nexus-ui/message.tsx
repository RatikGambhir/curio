"use client";

import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { Streamdown } from "streamdown";
import { cjk } from "@streamdown/cjk";
import { code } from "@streamdown/code";
import { math } from "@streamdown/math";
import { mermaid } from "@streamdown/mermaid";

import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { CodeBlock } from "@/components/nexus-ui/codeblock";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Kbd } from "@/components/ui/kbd";
import { cn } from "@/lib/utils";

const streamdownPlugins = { cjk, code, math, mermaid } as const;

// This project deliberately does not install `@tailwindcss/typography` (see the
// note in src/index.css), so the registry's `prose-*` variants would be inert.
// They are expressed here as explicit descendant selectors instead, which also
// re-adds the list/quote spacing that Tailwind's preflight resets.
const messageMarkdownProseClasses = [
  // Answers are read, not scanned, so they are set in the reading serif.
  "max-w-none font-serif text-foreground text-[1.0625rem] font-normal leading-[1.7]",
  // headings
  "[&_:is(h1,h2,h3,h4,h5,h6)]:mt-7 [&_:is(h1,h2,h3,h4,h5,h6)]:mb-2 [&_:is(h1,h2,h3,h4,h5,h6)]:font-medium [&_:is(h1,h2,h3,h4,h5,h6)]:leading-snug [&_:is(h1,h2,h3,h4,h5,h6):first-child]:mt-0",
  "[&_h1]:text-2xl [&_h2]:text-xl [&_h2]:tracking-[-0.01em] [&_h3]:text-lg [&_h4]:text-base [&_h5]:text-sm [&_h6]:text-sm",
  // heading links
  "[&_:is(h1,h2,h3,h4,h5,h6)_a]:text-inherit [&_:is(h1,h2,h3,h4,h5,h6)_a]:no-underline [&_:is(h1,h2,h3,h4,h5,h6)_a]:shadow-none",
  // body text
  "[&_p]:my-3 [&_p:first-child]:mt-0 [&_p:last-child]:mb-0",
  // links
  "[&_[data-streamdown=link]]:font-normal [&_[data-streamdown=link]]:text-primary [&_[data-streamdown=link]]:underline [&_[data-streamdown=link]]:decoration-primary/40 [&_[data-streamdown=link]]:underline-offset-3",
  // strong
  "[&_[data-streamdown=strong]]:font-semibold [&_[data-streamdown=strong]]:text-foreground",
  // lists
  "[&_ul]:my-3 [&_ul]:list-disc [&_ul]:pl-5 [&_ol]:my-3 [&_ol]:list-decimal [&_ol]:pl-5",
  "[&_li]:my-1 [&_li]:pl-1 [&_li]:marker:text-muted-foreground/50 [&_li_:is(ul,ol)]:my-1",
  // blockquotes and rules
  "[&_blockquote]:my-4 [&_blockquote]:border-l-2 [&_blockquote]:border-accent-brand/50 [&_blockquote]:pl-4 [&_blockquote]:italic [&_blockquote]:text-secondary-foreground",
  "[&_hr]:my-6 [&_hr]:border-border",
] as const;

type MessageFrom = "user" | "assistant";

type MessageContextValue = {
  from: MessageFrom;
};

const MessageContext = React.createContext<MessageContextValue | null>(null);

function useMessageContext() {
  return React.useContext(MessageContext);
}

type MessageProps = React.HTMLAttributes<HTMLDivElement> & {
  from: MessageFrom;
};

const Message = React.forwardRef<HTMLDivElement, MessageProps>(function Message(
  {
    className,
    from,
    children,
    "aria-label": ariaLabelProp,
    "aria-labelledby": ariaLabelledBy,
    ...props
  },
  ref,
) {
  const ariaLabel =
    ariaLabelProp ??
    (ariaLabelledBy == null
      ? from === "user"
        ? "User message"
        : "Assistant message"
      : undefined);

  return (
    <MessageContext.Provider value={{ from }}>
      <div
        ref={ref}
        data-slot="message"
        role="article"
        aria-label={ariaLabel}
        aria-labelledby={ariaLabelledBy}
        className={cn(
          "group/message flex w-full items-start gap-2",
          from === "user" ? "ms-auto max-w-[85%]" : "me-auto",
          className,
        )}
        {...props}
      >
        {children}
      </div>
    </MessageContext.Provider>
  );
});

type MessageStackProps = React.HTMLAttributes<HTMLDivElement>;

function MessageStack({ className, ...props }: MessageStackProps) {
  const ctx = useMessageContext();
  const from = ctx?.from ?? "assistant";

  return (
    <div
      data-slot="message-stack"
      className={cn(
        "flex w-full flex-col gap-2",
        from === "user" ? "items-end" : "items-start",
        className,
      )}
      {...props}
    />
  );
}

type MessageContentProps = React.HTMLAttributes<HTMLDivElement>;

function MessageContent({ className, ...props }: MessageContentProps) {
  const ctx = useMessageContext();
  const from = ctx?.from ?? "assistant";

  return (
    <div
      data-slot="message-content"
      className={cn(
        "text-foreground",
        from === "user"
          ? "w-fit whitespace-pre-wrap rounded-xl rounded-br-sm bg-secondary px-4 py-2.5 text-[0.9375rem] leading-6"
          : "mb-1 w-full bg-transparent",
        className,
      )}
      {...props}
    />
  );
}

type MessageMarkdownProps = React.ComponentProps<typeof Streamdown>;

function MessageMarkdown({
  className,
  components,
  ...props
}: MessageMarkdownProps) {
  const mergedComponents = React.useMemo(
    () => {
      const defaultComponents = {
        code: CodeBlock,
        inlineCode: ({
          children,
          className,
          ...props
        }: React.HTMLAttributes<HTMLElement>) => (
          <code
            className={cn(
              "rounded-sm border-none bg-secondary px-1.5 py-0.5 font-mono text-[0.8125em] font-normal",
              className,
            )}
            data-slot="message-markdown-inline-code"
            {...props}
          >
            {children}
          </code>
        ),
        table: (props: React.HTMLAttributes<HTMLTableElement>) => (
          <div
            data-slot="message-markdown-table-wrap"
            className={[
              "my-6 overflow-hidden rounded-lg border border-border font-sans",
              "[&_tbody_tr:first-child_td:first-child]:rounded-ss-xl",
              "[&_tbody_tr:first-child_td:last-child]:rounded-se-xl",
              "[&_tbody_tr:last-child_td:first-child]:rounded-es-xl",
              "[&_tbody_tr:last-child_td:last-child]:rounded-ee-xl",
            ].join(" ")}
          >
            <table
              data-slot="message-markdown-table"
              className="w-full border-separate border-spacing-0 border-none bg-secondary text-sm"
              {...props}
            />
          </div>
        ),
        th: (props: React.ThHTMLAttributes<HTMLTableCellElement>) => (
          <th
            data-slot="message-markdown-th"
            className="border-none px-4 py-2 text-start text-[0.8125rem] font-medium! text-muted-foreground!"
            {...props}
          />
        ),
        td: (props: React.TdHTMLAttributes<HTMLTableCellElement>) => (
          <td
            data-slot="message-markdown-td"
            className="border-0 border-border bg-card px-4 py-2.5 text-[0.8125rem] text-foreground [tr:not(:first-child)_&]:border-t"
            {...props}
          />
        ),
      };

      return {
        ...(defaultComponents as object),
        ...((components ?? {}) as object),
      };
    },
    [components],
  );

  return (
    <Streamdown
      data-slot="message-markdown"
      className={cn(
        ...messageMarkdownProseClasses,
        "[&>*:first-child]:mt-0 [&>*:last-child]:mb-0",
        className,
      )}
      components={mergedComponents as MessageMarkdownProps["components"]}
      shikiTheme={["github-light", "github-dark"]}
      plugins={streamdownPlugins}
      {...props}
    />
  );
}

type MessageActionsProps = React.HTMLAttributes<HTMLDivElement>;

function MessageActions({ className, ...props }: MessageActionsProps) {
  const ctx = useMessageContext();
  const from = ctx?.from ?? "assistant";

  return (
    <div
      data-slot="message-actions"
      className={cn(
        "flex w-full",
        from === "user" ? "justify-end" : "justify-start",
        className,
      )}
      {...props}
    />
  );
}

type MessageActionGroupProps = React.HTMLAttributes<HTMLDivElement>;

function MessageActionGroup({ className, ...props }: MessageActionGroupProps) {
  return (
    <div
      data-slot="message-action-group"
      className={cn("flex items-center gap-1", className)}
      {...props}
    />
  );
}

type MessageActionProps = React.HTMLAttributes<HTMLDivElement> & {
  asChild?: boolean;
  tooltip?:
    | string
    | {
        content?: string;
        side?: "top" | "right" | "bottom" | "left";
        shortcut?: string;
      };
};

function MessageAction({
  asChild = false,
  tooltip,
  ...props
}: MessageActionProps) {
  const Comp = asChild ? Slot : "div";
  const { content, side, shortcut } =
    typeof tooltip === "string" ? { content: tooltip } : tooltip ?? {};

  if (!content) {
    return <Comp data-slot="message-action" {...props} />;
  }

  return (
    <TooltipProvider delayDuration={200}>
      <Tooltip>
        <TooltipTrigger asChild>
          <Comp data-slot="message-action" {...props} />
        </TooltipTrigger>
        <TooltipContent side={side}>
          {content}
          {shortcut ? <Kbd className="rounded-md!">{shortcut}</Kbd> : null}
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

export type MessageAvatarProps = {
  src: string;
  alt?: string;
  fallback?: React.ReactNode;
  delayMs?: React.ComponentProps<typeof AvatarFallback>["delayMs"];
  size?: React.ComponentProps<typeof Avatar>["size"];
  className?: string;
};

function MessageAvatar({
  src,
  alt = "",
  fallback,
  delayMs,
  size,
  className,
}: MessageAvatarProps) {
  return (
    <Avatar
      data-slot="message-avatar"
      size={size}
      className={cn("size-7 shrink-0", className)}
    >
      <AvatarImage
        data-slot="message-avatar-image"
        src={src}
        alt={alt}
        className="my-0!"
      />
      <AvatarFallback
        data-slot="message-avatar-fallback"
        delayMs={delayMs}
        className="my-0! shrink-0"
      >
        {fallback}
      </AvatarFallback>
    </Avatar>
  );
}

export {
  Message,
  MessageStack,
  MessageContent,
  MessageMarkdown,
  MessageActions,
  MessageActionGroup,
  MessageAction,
  MessageAvatar,
};
