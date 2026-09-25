import { useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";

import { AuthLayout } from "@/components/auth/auth-layout";
import {
  ProfileSetupProgress,
  ProfileSetupStepper,
} from "@/components/profile-setup/profile-setup-stepper";
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
import { initialsFor } from "@/lib/initials";
import { validateEmail } from "@/lib/validators/auth";

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
    () => initialsFor(formData.name.trim() || formData.username.trim()),
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
    <AuthLayout
      statement="Let’s set up your notebook."
      aside={
        <ProfileSetupStepper
          currentStep={currentStep}
          steps={PROFILE_SETUP_STEPS}
        />
      }
    >
      <ProfileSetupProgress currentStep={currentStep} steps={PROFILE_SETUP_STEPS} />
      <p className="eyebrow mt-5 text-muted-foreground lg:mt-0">
        Profile setup · Step {currentStep} of {PROFILE_SETUP_STEPS.length}
      </p>
      <h1 className="mt-3 font-display text-display-md text-foreground">
        {currentStepMeta.title}
      </h1>
      <p className="mt-2 text-sm text-muted-foreground">
        {currentStepMeta.description} You can change these details at any time.
      </p>

      <div key={currentStep} className="rise-in mt-8">
        {renderStep()}
      </div>

      <div className="mt-10 flex items-center justify-between gap-3 border-t border-border pt-6">
        <Button
          type="button"
          variant="ghost"
          onClick={handleBack}
          disabled={currentStep === 1}
        >
          Back
        </Button>
        <div className="flex items-center gap-4">
          {currentStep < 4 ? (
            <Button type="button" variant="link" onClick={handleSkipToReview}>
              Skip to review
            </Button>
          ) : null}
          <Button type="button" onClick={handleContinue}>
            {currentStep === 4 ? "Finish setup" : "Continue"}
          </Button>
        </div>
      </div>
    </AuthLayout>
  );
}

export default ProfileSetupWizard;
