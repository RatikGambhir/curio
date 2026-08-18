import { Search } from "lucide-react";

import { Input } from "@/components/ui/input";

interface VaultSearchBarProps {
  value: string;
  onValueChange: (value: string) => void;
}

export function VaultSearchBar({ value, onValueChange }: VaultSearchBarProps) {
  return (
    <div className="relative">
      <Search className="text-muted-foreground pointer-events-none absolute left-4 top-1/2 size-5 -translate-y-1/2" />
      <Input
        type="search"
        value={value}
        onChange={(event) => onValueChange(event.target.value)}
        placeholder="Search questions and answers..."
        aria-label="Search questions and answers"
        className="h-12 rounded-lg border-border bg-card pl-12 pr-4 text-base"
      />
    </div>
  );
}
