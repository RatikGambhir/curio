import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

import type {
  ProfileSetupField,
  ProfileSetupFieldErrors,
  ProfileSetupFormData,
} from "../profile-setup.types";

type ContactStepProps = {
  formData: ProfileSetupFormData;
  errors: ProfileSetupFieldErrors;
  onFieldChange: (field: ProfileSetupField, value: string) => void;
};

export function ContactStep({
  formData,
  errors,
  onFieldChange,
}: ContactStepProps) {
  return (
    <div className="grid gap-5 sm:grid-cols-2">
      <div className="space-y-2 sm:col-span-2">
        <Label htmlFor="email">
          Email
          <span className="text-destructive">*</span>
        </Label>
        <Input
          id="email"
          type="email"
          value={formData.email}
          placeholder="you@curio.app"
          onChange={(event) => onFieldChange("email", event.target.value)}
          aria-invalid={Boolean(errors.email)}
          aria-describedby={errors.email ? "profile-email-error" : undefined}
        />
        {errors.email ? (
          <p id="profile-email-error" className="text-xs text-destructive">
            {errors.email}
          </p>
        ) : null}
      </div>

      <div className="space-y-2">
        <Label htmlFor="phone">Phone</Label>
        <Input
          id="phone"
          type="tel"
          value={formData.phone}
          placeholder="+1 (555) 123-4567"
          onChange={(event) => onFieldChange("phone", event.target.value)}
        />
      </div>

      <div className="space-y-2">
        <Label htmlFor="location">Location</Label>
        <Input
          id="location"
          value={formData.location}
          placeholder="Chicago, IL"
          onChange={(event) => onFieldChange("location", event.target.value)}
        />
      </div>
    </div>
  );
}
