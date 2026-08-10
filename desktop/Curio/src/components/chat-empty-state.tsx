import { ChatComposer } from "@/components/chat-composer";
import { cn } from "@/lib/utils";

type ChatEmptyStateProps = {
  className?: string;
  disabled?: boolean;
  onSubmit?: (text: string) => void;
};

export function ChatEmptyState({ className, disabled = false, onSubmit }: ChatEmptyStateProps) {
  return (
    <div
      className={cn(
        "mx-auto flex h-full w-full flex-col items-center justify-center px-4 pb-20 pt-8 md:px-8",
        className,
      )}
    >
      <div className="w-full max-w-3xl">
        <div className="mb-7 text-center">
          <h1 className="text-2xl font-normal tracking-[-0.025em] text-foreground md:text-[1.75rem]">
            What can I help you with?
          </h1>
          <p className="mt-2 text-sm font-normal text-muted-foreground">
            Start with a question, an idea, or something you want to make.
          </p>
        </div>
        <ChatComposer disabled={disabled} showShortcuts onSubmit={onSubmit} />
      </div>
    </div>
  );
}
