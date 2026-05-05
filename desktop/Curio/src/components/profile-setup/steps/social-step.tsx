import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

import type {
  ProfileSetupField,
  ProfileSetupFormData,
} from "../profile-setup.types";

type SocialStepProps = {
  formData: ProfileSetupFormData;
  onFieldChange: (field: ProfileSetupField, value: string) => void;
};

export function SocialStep({ formData, onFieldChange }: SocialStepProps) {
  return (
    <div className="grid gap-5 sm:grid-cols-2">
      <div className="space-y-2 sm:col-span-2">
        <Label htmlFor="website">Website</Label>
        <Input
          id="website"
          type="url"
          value={formData.website}
          placeholder="https://your-site.com"
          onChange={(event) => onFieldChange("website", event.target.value)}
        />
      </div>

      <div className="space-y-2">
        <Label htmlFor="twitter">X (Twitter)</Label>
        <Input
          id="twitter"
          value={formData.twitter}
          placeholder="@curio"
          onChange={(event) => onFieldChange("twitter", event.target.value)}
        />
      </div>

      <div className="space-y-2">
        <Label htmlFor="linkedin">LinkedIn</Label>
        <Input
          id="linkedin"
          value={formData.linkedin}
          placeholder="linkedin.com/in/your-handle"
          onChange={(event) => onFieldChange("linkedin", event.target.value)}
        />
      </div>
    </div>
  );
}
