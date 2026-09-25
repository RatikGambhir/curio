import { cn } from "@/lib/utils";

interface VaultCategoryFilterProps {
  categories: { name: string; count: number }[];
  selectedCategory: string;
  onCategoryChange: (category: string) => void;
}

/* Categories as a row of index tabs with their counts, always visible: there
   are few enough that hiding them behind a toggle only cost a click. */
export function VaultCategoryFilter({
  categories,
  selectedCategory,
  onCategoryChange,
}: VaultCategoryFilterProps) {
  return (
    <div
      role="radiogroup"
      aria-label="Filter by category"
      className="no-scrollbar -mx-1 flex gap-1 overflow-x-auto px-1"
    >
      {categories.map(({ name, count }) => {
        const isActive = name === selectedCategory;

        return (
          <label key={name} className="shrink-0 cursor-pointer">
            <input
              type="radio"
              name="vault-category"
              value={name}
              checked={isActive}
              onChange={() => onCategoryChange(name)}
              className="peer sr-only"
            />
            <span
              className={cn(
                "flex h-8 items-center gap-2 rounded-md px-3 text-sm transition-colors duration-150",
                "text-muted-foreground hover:bg-accent hover:text-foreground",
                "peer-checked:bg-foreground peer-checked:text-background",
                "peer-focus-visible:outline-2 peer-focus-visible:outline-offset-2 peer-focus-visible:outline-ring",
              )}
            >
              {name}
              <span className="font-mono text-2xs tabular-nums opacity-70">
                {count}
              </span>
            </span>
          </label>
        );
      })}
    </div>
  );
}
