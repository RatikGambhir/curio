import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";

import type { ProfileSetupFormData } from "../profile-setup.types";

type ReviewStepProps = {
  formData: ProfileSetupFormData;
  avatarInitials: string;
};

const NOT_PROVIDED = "Not provided";

const renderValue = (value: string) => value.trim() || NOT_PROVIDED;
const renderUsername = (value: string) =>
  value.trim() ? `@${value.trim()}` : NOT_PROVIDED;

function ReviewGroup({
  title,
  rows,
}: {
  title: string;
  rows: { label: string; value: string }[];
}) {
  return (
    <section className="border-t border-border pt-4">
      <h3 className="eyebrow text-muted-foreground">{title}</h3>
      <dl className="mt-3 grid grid-cols-[6.5rem_minmax(0,1fr)] gap-x-4 gap-y-2 text-sm">
        {rows.map((row) => (
          <div key={row.label} className="contents">
            <dt className="text-muted-foreground">{row.label}</dt>
            <dd
              className={
                row.value === NOT_PROVIDED
                  ? "italic text-muted-foreground"
                  : "break-words text-foreground"
              }
            >
              {row.value}
            </dd>
          </div>
        ))}
      </dl>
    </section>
  );
}

export function ReviewStep({ formData, avatarInitials }: ReviewStepProps) {
  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <Avatar className="size-14 rounded-lg">
          {formData.avatar ? (
            <AvatarImage src={formData.avatar} alt="Profile avatar preview" />
          ) : null}
          <AvatarFallback className="rounded-lg bg-accent-subtle font-mono text-sm text-foreground">
            {avatarInitials}
          </AvatarFallback>
        </Avatar>

        <div className="min-w-0">
          <p className="truncate font-display text-display-sm text-foreground">
            {renderValue(formData.name)}
          </p>
          <p className="truncate text-sm text-muted-foreground">
            {renderUsername(formData.username)}
          </p>
        </div>
      </div>

      <ReviewGroup
        title="About"
        rows={[{ label: "Bio", value: renderValue(formData.bio) }]}
      />
      <ReviewGroup
        title="Contact"
        rows={[
          { label: "Email", value: renderValue(formData.email) },
          { label: "Phone", value: renderValue(formData.phone) },
          { label: "Location", value: renderValue(formData.location) },
        ]}
      />
      <ReviewGroup
        title="Links"
        rows={[
          { label: "Website", value: renderValue(formData.website) },
          { label: "X", value: renderValue(formData.twitter) },
          { label: "LinkedIn", value: renderValue(formData.linkedin) },
        ]}
      />
    </div>
  );
}
