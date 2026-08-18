import { useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";

import curioLogo from "@/assets/curio-logo.png";
import { ProfileSetupStepper } from "@/components/profile-setup/profile-setup-stepper";
import {
  INITIAL_PROFILE_SETUP_FORM_DATA,
  PROFILE_SETUP_STEPS,
  type ProfileSetupField,
  type ProfileSetupFieldErrors,
  type ProfileSetupFormData,
  type ProfileSetupStep,
} from "@/components/profile-setup/profile-setup.types";
import { BasicInfoStep } from "@/components/profile-setup/steps/basic-info-step";
import { ContactStep } from "@/components/profile-setup/steps/contact-step";
import { ReviewStep } from "@/components/profile-setup/steps/review-step";
import { SocialStep } from "@/components/profile-setup/steps/social-step";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { validateEmail } from "@/lib/validators/auth";

function getInitials(name: string, username: string): string {
  const source = name.trim() || username.trim() || "Curio Member";
  const parts = source.split(/\s+/).filter(Boolean);

  if (parts.length === 0) {
    return "CU";
  }

  const joined = parts.slice(0, 2).map((part) => part[0]).join("");
  return joined.toUpperCase();
}

function getStepErrors(
  step: ProfileSetupStep,
  formData: ProfileSetupFormData,
): ProfileSetupFieldErrors {
  const errors: ProfileSetupFieldErrors = {};

  if (step === 1) {
    if (!formData.name.trim()) {
      errors.name = "Name is required.";
    }

    if (!formData.username.trim()) {
      errors.username = "Username is required.";
    }
  }

  if (step === 2) {
    const emailError = validateEmail(formData.email);
    if (emailError) {
      errors.email = emailError;
    }
  }

  return errors;
}

function getNextStep(step: ProfileSetupStep): ProfileSetupStep {
  return Math.min(step + 1, 4) as ProfileSetupStep;
}

function getPreviousStep(step: ProfileSetupStep): ProfileSetupStep {
  return Math.max(step - 1, 1) as ProfileSetupStep;
}

function ProfileSetupWizard() {
  const navigate = useNavigate();
  const [currentStep, setCurrentStep] = useState<ProfileSetupStep>(1);
  const [hasAttemptedContinue, setHasAttemptedContinue] = useState(false);
  const [formData, setFormData] = useState<ProfileSetupFormData>(
    INITIAL_PROFILE_SETUP_FORM_DATA,
  );

  const avatarInitials = useMemo(
    () => getInitials(formData.name, formData.username),
    [formData.name, formData.username],
  );

  const stepErrors = useMemo(
    () => getStepErrors(currentStep, formData),
    [currentStep, formData],
  );

  const currentStepMeta =
    PROFILE_SETUP_STEPS.find((step) => step.id === currentStep) ??
    PROFILE_SETUP_STEPS[0];

  const visibleErrors = hasAttemptedContinue ? stepErrors : {};

  const handleFieldChange = (field: ProfileSetupField, value: string) => {
    setFormData((prev) => ({
      ...prev,
      [field]: value,
    }));
  };

  const handleBack = () => {
    if (currentStep === 1) {
      return;
    }

    setCurrentStep((prev) => getPreviousStep(prev));
    setHasAttemptedContinue(false);
  };

  const handleContinue = () => {
    if (currentStep === 4) {
      navigate("/home");
      return;
    }

    const errors = getStepErrors(currentStep, formData);
    if (Object.keys(errors).length > 0) {
      setHasAttemptedContinue(true);
      return;
    }

    setCurrentStep((prev) => getNextStep(prev));
    setHasAttemptedContinue(false);
  };

  const handleSkipToReview = () => {
    setCurrentStep(4);
    setHasAttemptedContinue(false);
  };

  const renderStep = () => {
    if (currentStep === 1) {
      return (
        <BasicInfoStep
          formData={formData}
          errors={visibleErrors}
          avatarInitials={avatarInitials}
          onFieldChange={handleFieldChange}
        />
      );
    }

    if (currentStep === 2) {
      return (
        <ContactStep
          formData={formData}
          errors={visibleErrors}
          onFieldChange={handleFieldChange}
        />
      );
    }

    if (currentStep === 3) {
      return <SocialStep formData={formData} onFieldChange={handleFieldChange} />;
    }

    return <ReviewStep formData={formData} avatarInitials={avatarInitials} />;
  };

  return (
    <div className="min-h-screen bg-muted/30 px-4 py-8 sm:py-10">
      <div className="mx-auto flex w-full max-w-3xl flex-col items-center gap-6">
        <img
          src={curioLogo}
          alt="Curio Logo"
          className="h-auto w-48 sm:w-56"
          style={{ mixBlendMode: "multiply" }}
        />

        <div className="w-full space-y-5">
          <header className="space-y-2 text-center">
            <h1 className="text-3xl font-semibold text-foreground">
              Complete Your Profile
            </h1>
            <p className="text-sm text-muted-foreground">
              Tell us a little about yourself. You can update these details at
              any time.
            </p>
          </header>

          <ProfileSetupStepper currentStep={currentStep} steps={PROFILE_SETUP_STEPS} />

          <Card className="w-full rounded-2xl border-border/80 shadow-lg">
            <CardHeader className="space-y-1 border-b border-border">
              <CardTitle className="text-xl">{currentStepMeta.title}</CardTitle>
              <CardDescription>{currentStepMeta.description}</CardDescription>
            </CardHeader>

            <CardContent className="pt-6">{renderStep()}</CardContent>

            <CardFooter className="flex flex-col gap-3 border-t border-border pt-6">
              <div className="flex w-full items-center justify-between gap-3">
                <Button
                  type="button"
                  variant="outline"
                  onClick={handleBack}
                  disabled={currentStep === 1}
                >
                  Back
                </Button>
                <Button type="button" onClick={handleContinue}>
                  {currentStep === 4 ? "Complete Setup" : "Continue"}
                </Button>
              </div>

              {currentStep < 4 ? (
                <button
                  type="button"
                  onClick={handleSkipToReview}
                  className="self-end text-sm font-medium text-primary underline-offset-4 hover:underline"
                >
                  Skip to review
                </button>
              ) : null}
            </CardFooter>
          </Card>
        </div>
      </div>
    </div>
  );
}

export default ProfileSetupWizard;
