import { ChevronDown, SlidersHorizontal } from "lucide-react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface VaultFilterBarProps {
  showFilters: boolean;
  categories: string[];
  selectedCategory: string;
  onToggleFilters: () => void;
  onCategoryChange: (category: string) => void;
}

export function VaultFilterBar({
  showFilters,
  categories,
  selectedCategory,
  onToggleFilters,
  onCategoryChange,
}: VaultFilterBarProps) {
  return (
    <div className="flex flex-wrap items-start gap-3">
      <Button
        type="button"
        variant="secondary"
        onClick={onToggleFilters}
        className="h-10 rounded-lg px-4"
      >
        <SlidersHorizontal className="size-4" />
        Filters
        <ChevronDown
          className={cn(
            "size-4 transition-transform",
            showFilters && "rotate-180",
          )}
        />
      </Button>

      {showFilters ? (
        <div className="flex flex-wrap items-center gap-2">
          {categories.map((category) => {
            const isActive = category === selectedCategory;

            return (
              <Button
                key={category}
                type="button"
                size="sm"
                variant={isActive ? "default" : "secondary"}
                onClick={() => onCategoryChange(category)}
                className="rounded-md"
              >
                {category}
              </Button>
            );
          })}
        </div>
      ) : null}
    </div>
  );
}
