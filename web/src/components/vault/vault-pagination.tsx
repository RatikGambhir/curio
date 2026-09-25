import { ChevronLeft, ChevronRight } from "lucide-react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface VaultPaginationProps {
  currentPage: number;
  totalPages: number;
  totalItems: number;
  startIndex: number;
  endIndex: number;
  onPageChange: (page: number) => void;
}

export function VaultPagination({
  currentPage,
  totalPages,
  totalItems,
  startIndex,
  endIndex,
  onPageChange,
}: VaultPaginationProps) {
  if (totalPages <= 1) {
    return null;
  }

  return (
    <nav
      aria-label="Vault pages"
      className="mt-8 flex flex-col-reverse items-center justify-between gap-4 sm:flex-row"
    >
      <p className="font-mono text-xs tabular-nums text-muted-foreground">
        {startIndex + 1}–{Math.min(endIndex, totalItems)} of {totalItems}
      </p>

      <div className="flex items-center gap-1">
        <Button
          type="button"
          variant="ghost"
          size="sm"
          onClick={() => onPageChange(currentPage - 1)}
          disabled={currentPage === 1}
        >
          <ChevronLeft aria-hidden="true" />
          Previous
        </Button>

        <ol className="flex items-center gap-1">
          {Array.from({ length: totalPages }, (_, index) => {
            const page = index + 1;
            const isActive = currentPage === page;

            return (
              <li key={page}>
                <button
                  type="button"
                  onClick={() => onPageChange(page)}
                  aria-current={isActive ? "page" : undefined}
                  aria-label={`Page ${page}`}
                  className={cn(
                    "focus-ring flex size-8 items-center justify-center rounded-md font-mono text-xs tabular-nums transition-colors",
                    isActive
                      ? "bg-foreground text-background"
                      : "text-muted-foreground hover:bg-accent hover:text-foreground",
                  )}
                >
                  {page}
                </button>
              </li>
            );
          })}
        </ol>

        <Button
          type="button"
          variant="ghost"
          size="sm"
          onClick={() => onPageChange(currentPage + 1)}
          disabled={currentPage === totalPages}
        >
          Next
          <ChevronRight aria-hidden="true" />
        </Button>
      </div>
    </nav>
  );
}
