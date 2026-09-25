import { ArrowDown, ArrowUp, FileImage, FileText, Inbox } from "lucide-react"

import { SettingsSection } from "@/components/settings/settings-section"
import {
  ATTACHMENT_FILTER_OPTIONS,
  type AttachmentFilter,
  type AttachmentRecord,
  type AttachmentSortDirection,
} from "@/components/settings/settings.types"
import { EmptyState } from "@/components/ui/empty-state"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"

type AttachmentsTabProps = {
  selectedFilter: AttachmentFilter
  onFilterChange: (value: AttachmentFilter) => void
  attachments?: AttachmentRecord[]
  sortDirection?: AttachmentSortDirection
  onSortDirectionChange?: (direction: AttachmentSortDirection) => void
}

const GRID = "grid grid-cols-[2rem_minmax(0,1fr)_8rem] items-center gap-3 sm:grid-cols-[2rem_minmax(0,1fr)_11rem]"

export function AttachmentsTab({
  selectedFilter,
  onFilterChange,
  attachments = [],
  sortDirection = "desc",
  onSortDirectionChange,
}: AttachmentsTabProps) {
  const hasAttachments = attachments.length > 0
  const SortIcon = sortDirection === "asc" ? ArrowUp : ArrowDown

  return (
    <SettingsSection
      title="Attachments"
      description="Every file uploaded to your chats. Filter by type and review when each was added."
      aside={
        <Select
          value={selectedFilter}
          onValueChange={(value) => onFilterChange(value as AttachmentFilter)}
        >
          <SelectTrigger aria-label="Filter attachments" className="w-40">
            <SelectValue placeholder="All files" />
          </SelectTrigger>
          <SelectContent align="end">
            {ATTACHMENT_FILTER_OPTIONS.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      }
    >
      <div className="overflow-hidden rounded-lg border border-border">
        <div className={`${GRID} border-b border-border bg-secondary px-4 py-2.5`}>
          <input
            type="checkbox"
            disabled={!hasAttachments}
            aria-label="Select all attachments"
            className="size-4 rounded border border-border accent-primary disabled:cursor-not-allowed disabled:opacity-50"
          />
          <span className="eyebrow text-muted-foreground">Name</span>
          <button
            type="button"
            onClick={() =>
              onSortDirectionChange?.(sortDirection === "asc" ? "desc" : "asc")
            }
            disabled={!onSortDirectionChange}
            aria-label={`Sort by date added, ${sortDirection === "asc" ? "oldest" : "newest"} first`}
            className="focus-ring eyebrow inline-flex items-center gap-1 justify-self-start rounded-sm text-muted-foreground transition-colors hover:text-foreground"
          >
            Added
            <SortIcon className="size-3.5" aria-hidden="true" />
          </button>
        </div>

        {hasAttachments ? (
          <ul>
            {attachments.map((attachment) => (
              <li
                key={attachment.id}
                className={`${GRID} border-b border-border bg-card px-4 py-3 last:border-b-0`}
              >
                <input
                  type="checkbox"
                  aria-label={`Select ${attachment.name}`}
                  className="size-4 rounded border border-border accent-primary"
                />
                <div className="flex min-w-0 items-center gap-2">
                  {attachment.type === "image" ? (
                    <FileImage className="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
                  ) : (
                    <FileText className="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
                  )}
                  <span className="truncate text-sm text-foreground">
                    {attachment.name}
                  </span>
                </div>
                <span className="font-mono text-xs tabular-nums text-muted-foreground">
                  {attachment.createdAt}
                </span>
              </li>
            ))}
          </ul>
        ) : (
          <EmptyState icon={Inbox} title="No attachments yet" headingLevel={3} className="bg-card py-16">
            Files you add to chats will be listed here.
          </EmptyState>
        )}
      </div>
    </SettingsSection>
  )
}
