import { ChevronLeft, ChevronRight } from "lucide-react";

import { Button } from "@/components/ui/button";

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
  if (totalItems <= 5) {
    return null;
  }

  return (
    <div className="mt-8 flex flex-col items-start justify-between gap-4 md:flex-row md:items-center">
      <div className="text-muted-foreground text-sm">
        Showing {startIndex + 1}-{Math.min(endIndex, totalItems)} of {totalItems}
      </div>

      <div className="flex items-center gap-2">
        <Button
          type="button"
          variant="outline"
          onClick={() => onPageChange(Math.max(1, currentPage - 1))}
          disabled={currentPage === 1}
          className="h-10 rounded-lg"
        >
          <ChevronLeft className="size-4" />
          Previous
        </Button>

        <div className="flex items-center gap-1">
          {Array.from({ length: totalPages }, (_, index) => {
            const page = index + 1;
            const isActive = currentPage === page;

            return (
              <Button
                key={page}
                type="button"
                size="icon"
                variant={isActive ? "default" : "outline"}
                onClick={() => onPageChange(page)}
                className="size-10 rounded-lg"
              >
                {page}
              </Button>
            );
          })}
        </div>

        <Button
          type="button"
          variant="outline"
          onClick={() => onPageChange(Math.min(totalPages, currentPage + 1))}
          disabled={currentPage === totalPages}
          className="h-10 rounded-lg"
        >
          Next
          <ChevronRight className="size-4" />
        </Button>
      </div>
    </div>
  );
}
