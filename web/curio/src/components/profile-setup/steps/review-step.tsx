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

export function ReviewStep({ formData, avatarInitials }: ReviewStepProps) {
  return (
    <div className="space-y-5">
      <div className="flex items-center gap-4 rounded-xl border border-border bg-muted/40 p-4">
        <Avatar className="size-14 border border-border">
          {formData.avatar ? (
            <AvatarImage src={formData.avatar} alt="Profile avatar preview" />
          ) : null}
          <AvatarFallback className="bg-secondary text-secondary-foreground">
            {avatarInitials}
          </AvatarFallback>
        </Avatar>

        <div className="min-w-0">
          <p className="text-base font-semibold text-foreground">
            {renderValue(formData.name)}
          </p>
          <p className="text-sm text-muted-foreground">
            {renderUsername(formData.username)}
          </p>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <section className="space-y-2 rounded-xl border border-border p-4">
          <h3 className="text-sm font-semibold text-foreground">Basic info</h3>
          <p className="text-sm text-muted-foreground">
            Bio: {renderValue(formData.bio)}
          </p>
        </section>

        <section className="space-y-2 rounded-xl border border-border p-4">
          <h3 className="text-sm font-semibold text-foreground">Contact</h3>
          <p className="text-sm text-muted-foreground">
            Email: {renderValue(formData.email)}
          </p>
          <p className="text-sm text-muted-foreground">
            Phone: {renderValue(formData.phone)}
          </p>
          <p className="text-sm text-muted-foreground">
            Location: {renderValue(formData.location)}
          </p>
        </section>

        <section className="space-y-2 rounded-xl border border-border p-4 md:col-span-2">
          <h3 className="text-sm font-semibold text-foreground">Social links</h3>
          <p className="text-sm text-muted-foreground">
            Website: {renderValue(formData.website)}
          </p>
          <p className="text-sm text-muted-foreground">
            X: {renderValue(formData.twitter)}
          </p>
          <p className="text-sm text-muted-foreground">
            LinkedIn: {renderValue(formData.linkedin)}
          </p>
        </section>
      </div>
    </div>
  );
}
