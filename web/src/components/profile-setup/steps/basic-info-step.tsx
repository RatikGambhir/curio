import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";

import type {
  ProfileSetupField,
  ProfileSetupFieldErrors,
  ProfileSetupFormData,
} from "../profile-setup.types";

type BasicInfoStepProps = {
  formData: ProfileSetupFormData;
  errors: ProfileSetupFieldErrors;
  avatarInitials: string;
  onFieldChange: (field: ProfileSetupField, value: string) => void;
};

export function BasicInfoStep({
  formData,
  errors,
  avatarInitials,
  onFieldChange,
}: BasicInfoStepProps) {
  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start">
        <Avatar className="size-16 rounded-lg">
          {formData.avatar ? (
            <AvatarImage src={formData.avatar} alt="Profile avatar preview" />
          ) : null}
          <AvatarFallback className="rounded-lg bg-accent-subtle font-mono text-sm text-foreground">
            {avatarInitials}
          </AvatarFallback>
        </Avatar>

        <div className="flex-1 space-y-2">
          <Label htmlFor="avatar">Avatar URL</Label>
          <Input
            id="avatar"
            type="url"
            value={formData.avatar}
            placeholder="https://example.com/avatar.jpg"
            onChange={(event) => onFieldChange("avatar", event.target.value)}
          />
          <p className="text-xs text-muted-foreground">
            Optional. Paste an image URL to preview your profile photo.
          </p>
        </div>
      </div>

      <div className="grid gap-5 sm:grid-cols-2">
        <div className="space-y-2">
          <Label htmlFor="name">
            Full name
            <span className="text-destructive" aria-hidden="true">*</span>
          </Label>
          <Input
            id="name"
            value={formData.name}
            placeholder="Curio Member"
            onChange={(event) => onFieldChange("name", event.target.value)}
            aria-invalid={Boolean(errors.name)}
            aria-required="true"
            aria-describedby={errors.name ? "profile-name-error" : undefined}
          />
          {errors.name ? (
            <p id="profile-name-error" className="text-xs text-destructive">
              {errors.name}
            </p>
          ) : null}
        </div>

        <div className="space-y-2">
          <Label htmlFor="username">
            Username
            <span className="text-destructive" aria-hidden="true">*</span>
          </Label>
          <Input
            id="username"
            value={formData.username}
            placeholder="curio_member"
            onChange={(event) => onFieldChange("username", event.target.value)}
            aria-invalid={Boolean(errors.username)}
            aria-required="true"
            aria-describedby={
              errors.username ? "profile-username-error" : undefined
            }
          />
          {errors.username ? (
            <p id="profile-username-error" className="text-xs text-destructive">
              {errors.username}
            </p>
          ) : null}
        </div>
      </div>

      <div className="space-y-2">
        <Label htmlFor="bio">Bio</Label>
        <Textarea
          id="bio"
          value={formData.bio}
          placeholder="Tell people a little about your interests and expertise."
          className="min-h-28"
          onChange={(event) => onFieldChange("bio", event.target.value)}
        />
      </div>
    </div>
  );
}
