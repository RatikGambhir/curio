export type ProfileSetupStep = 1 | 2 | 3 | 4;

export type ProfileSetupFormData = {
  name: string;
  username: string;
  avatar: string;
  bio: string;
  email: string;
  phone: string;
  location: string;
  website: string;
  twitter: string;
  linkedin: string;
};

export type ProfileSetupField = keyof ProfileSetupFormData;
export type ProfileSetupFieldErrors = Partial<
  Record<ProfileSetupField, string>
>;

export type ProfileSetupStepMeta = {
  id: ProfileSetupStep;
  title: string;
  description: string;
  shortLabel: string;
};

export const PROFILE_SETUP_STEPS: readonly ProfileSetupStepMeta[] = [
  {
    id: 1,
    title: "Basic Info",
    description: "Add your profile basics and an optional avatar.",
    shortLabel: "Basic",
  },
  {
    id: 2,
    title: "Contact",
    description: "Share your contact details for collaborators.",
    shortLabel: "Contact",
  },
  {
    id: 3,
    title: "Social",
    description: "Add public links for your online presence.",
    shortLabel: "Social",
  },
  {
    id: 4,
    title: "Review",
    description: "Confirm everything looks right before continuing.",
    shortLabel: "Review",
  },
];

export const INITIAL_PROFILE_SETUP_FORM_DATA: ProfileSetupFormData = {
  name: "",
  username: "",
  avatar: "",
  bio: "",
  email: "",
  phone: "",
  location: "",
  website: "",
  twitter: "",
  linkedin: "",
};
