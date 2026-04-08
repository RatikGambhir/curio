import type { QAPair } from "@/components/vault/vault.types";
import { VaultResultRow } from "@/components/vault/vault-result-row";

interface VaultResultsListProps {
  items: QAPair[];
}

export function VaultResultsList({ items }: VaultResultsListProps) {
  if (items.length === 0) {
    return (
      <div className="rounded-lg border border-border bg-card py-16 text-center">
        <p className="text-muted-foreground">No results found</p>
        <p className="text-muted-foreground mt-2 text-sm">
          Try adjusting your search or filters.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {items.map((item) => (
        <VaultResultRow key={item.id} item={item} />
      ))}
    </div>
  );
}
