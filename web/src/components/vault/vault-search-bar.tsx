import { Search, X } from "lucide-react";

import { fieldClassName } from "@/components/ui/field-styles";
import { cn } from "@/lib/utils";

interface VaultSearchBarProps {
  value: string;
  onValueChange: (value: string) => void;
}

export function VaultSearchBar({ value, onValueChange }: VaultSearchBarProps) {
  return (
    <div className="relative">
      <Search
        className="pointer-events-none absolute left-4 top-1/2 size-[1.125rem] -translate-y-1/2 text-muted-foreground"
        aria-hidden="true"
      />
      <input
        type="search"
        value={value}
        onChange={(event) => onValueChange(event.target.value)}
        placeholder="Search questions and answers"
        aria-label="Search questions and answers"
        className={cn(
          fieldClassName,
          "h-12 pl-11 pr-11 text-base md:text-[0.9375rem] [&::-webkit-search-cancel-button]:hidden",
        )}
      />
      {value ? (
        <button
          type="button"
          onClick={() => onValueChange("")}
          className="focus-ring absolute right-2 top-1/2 flex size-8 -translate-y-1/2 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
        >
          <X className="size-4" aria-hidden="true" />
          <span className="sr-only">Clear search</span>
        </button>
      ) : null}
    </div>
  );
}
