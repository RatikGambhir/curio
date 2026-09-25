import { SearchX } from "lucide-react";

import type { QAPair } from "@/components/vault/vault.types";
import { VaultResultRow } from "@/components/vault/vault-result-row";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/ui/empty-state";

interface VaultResultsListProps {
  items: QAPair[];
  query: string;
  onClearFilters: () => void;
}

export function VaultResultsList({
  items,
  query,
  onClearFilters,
}: VaultResultsListProps) {
  if (items.length === 0) {
    return (
      <EmptyState
        icon={SearchX}
        title="Nothing in the vault matches"
        action={
          <Button type="button" variant="outline" onClick={onClearFilters}>
            Clear search and filters
          </Button>
        }
        className="border-t border-border py-20"
      >
        Try a shorter phrase, or look across every category.
      </EmptyState>
    );
  }

  return (
    <div className="border-t border-border">
      {items.map((item) => (
        <VaultResultRow key={item.id} item={item} query={query} />
      ))}
    </div>
  );
}
