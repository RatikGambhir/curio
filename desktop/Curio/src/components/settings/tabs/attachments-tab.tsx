import {
  ATTACHMENT_FILTER_OPTIONS,
  type AttachmentFilter,
  type AttachmentRecord,
  type AttachmentSortDirection,
} from "@/components/settings/settings.types";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ArrowUpDown, FileImage, FileText, Inbox } from "lucide-react";

type AttachmentsTabProps = {
  selectedFilter: AttachmentFilter;
  onFilterChange: (value: AttachmentFilter) => void;
  attachments?: AttachmentRecord[];
  sortDirection?: AttachmentSortDirection;
  onSortDirectionChange?: (direction: AttachmentSortDirection) => void;
};

export function AttachmentsTab({
  selectedFilter,
  onFilterChange,
  attachments = [],
  sortDirection = "desc",
  onSortDirectionChange,
}: AttachmentsTabProps) {
  const hasAttachments = attachments.length > 0;

  const handleSortClick = () => {
    if (!onSortDirectionChange) {
      return;
    }

    onSortDirectionChange(sortDirection === "asc" ? "desc" : "asc");
  };

  return (
    <section className="rounded-[2rem] border border-border bg-card p-6 shadow-md sm:p-8">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h2 className="text-2xl font-bold text-card-foreground">Attachments</h2>
          <p className="mt-2 max-w-xl text-sm text-muted-foreground">
            Manage every file that has been uploaded to your chats. Filter by file type and
            review when each item was created.
          </p>
        </div>

        <Select
          value={selectedFilter}
          onValueChange={(value) => onFilterChange(value as AttachmentFilter)}
        >
          <SelectTrigger className="w-[11.25rem] rounded-full border-none bg-accent text-accent-foreground shadow-xs hover:bg-accent/85">
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
      </div>

      <div className="mt-8 overflow-hidden rounded-2xl border border-border bg-secondary/70">
        <div className="grid grid-cols-[2.5rem_minmax(0,1fr)_8.5rem] items-center gap-3 border-b border-border px-4 py-3 text-sm font-semibold text-muted-foreground sm:grid-cols-[2.5rem_minmax(0,1fr)_11rem]">
          <input
            type="checkbox"
            disabled={!hasAttachments}
            aria-label="Select all attachments"
            className="size-4 rounded border border-border accent-primary disabled:cursor-not-allowed"
          />
          <span>Name</span>
          <button
            type="button"
            onClick={handleSortClick}
            className="inline-flex items-center justify-start gap-1 text-left text-sm font-semibold text-muted-foreground hover:text-foreground"
          >
            Created
            <ArrowUpDown className="size-4" />
          </button>
        </div>

        {hasAttachments ? (
          <ul>
            {attachments.map((attachment) => (
              <li
                key={attachment.id}
                className="grid grid-cols-[2.5rem_minmax(0,1fr)_8.5rem] items-center gap-3 border-b border-border/70 bg-card px-4 py-3 last:border-b-0 sm:grid-cols-[2.5rem_minmax(0,1fr)_11rem]"
              >
                <input
                  type="checkbox"
                  aria-label={`Select ${attachment.name}`}
                  className="size-4 rounded border border-border accent-primary"
                />
                <div className="flex min-w-0 items-center gap-2">
                  {attachment.type === "image" ? (
                    <FileImage className="size-4 text-muted-foreground" />
                  ) : (
                    <FileText className="size-4 text-muted-foreground" />
                  )}
                  <span className="truncate text-sm font-medium text-card-foreground">
                    {attachment.name}
                  </span>
                </div>
                <span className="text-sm text-muted-foreground">{attachment.createdAt}</span>
              </li>
            ))}
          </ul>
        ) : (
          <div className="flex min-h-[18rem] items-center justify-center bg-card px-6 py-10">
            <div className="flex max-w-sm flex-col items-center text-center">
              <div className="flex size-14 items-center justify-center rounded-2xl bg-accent text-accent-foreground">
                <Inbox className="size-6" />
              </div>
              <h3 className="mt-4 text-xl font-bold text-card-foreground">No attachments found.</h3>
              <p className="mt-2 text-sm text-muted-foreground">
                Attachments from your chats will appear here as they are added.
              </p>
            </div>
          </div>
        )}
      </div>
    </section>
  );
}
