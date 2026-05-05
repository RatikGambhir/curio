import { Card } from "@/components/ui/card";
import type { QAPair } from "@/components/vault/vault.types";

interface VaultResultRowProps {
  item: QAPair;
}

export function VaultResultRow({ item }: VaultResultRowProps) {
  return (
    <Card className="h-auto gap-0 rounded-lg border-border px-5 py-5 shadow-sm transition-all hover:border-primary/30 hover:shadow-md">
      <div className="grid h-auto grid-cols-1 items-start gap-4 lg:grid-cols-2 lg:gap-6">
        <div className="space-y-2">
          <div className="flex items-start justify-between gap-3">
            <h3 className="whitespace-normal break-words text-xl font-semibold leading-tight text-foreground">
              {item.question}
            </h3>
            <span className="bg-muted text-muted-foreground shrink-0 rounded px-2 py-0.5 text-xs font-medium">
              {item.category}
            </span>
          </div>
          <p className="text-muted-foreground text-xs">{item.date}</p>
        </div>

        <div className="h-auto lg:border-l lg:border-border lg:pl-6">
          <p className="text-muted-foreground whitespace-normal break-words text-lg leading-relaxed">
            {item.answer}
          </p>
        </div>
      </div>
    </Card>
  );
}
