import { Check } from "lucide-react";

import {
  Stepper,
  StepperConnector,
  StepperDescription,
  StepperIndicator,
  StepperItem,
  StepperLabel,
  StepperList,
} from "@/components/ui/stepper";

import type {
  ProfileSetupStep,
  ProfileSetupStepMeta,
} from "./profile-setup.types";

type ProfileSetupStepperProps = {
  currentStep: ProfileSetupStep;
  steps: readonly ProfileSetupStepMeta[];
};

export function ProfileSetupStepper({
  currentStep,
  steps,
}: ProfileSetupStepperProps) {
  return (
    <Stepper aria-label="Profile setup progress">
      <StepperList>
        {steps.map((step, index) => {
          const isCompleted = step.id < currentStep;
          const isActive = step.id === currentStep;
          const isConnectorComplete = currentStep > step.id;
          const labelState = isActive
            ? "current"
            : isCompleted
              ? "completed"
              : "upcoming";
          const indicatorState = isCompleted
            ? "completed"
            : isActive
              ? "active"
              : "upcoming";
          const connectorState = isConnectorComplete ? "completed" : "upcoming";

          return (
            <StepperItem key={step.id}>
              <div className="flex min-w-0 flex-col items-center gap-2 text-center">
                <StepperIndicator state={indicatorState}>
                  {isCompleted ? <Check className="size-4" /> : step.id}
                </StepperIndicator>
                <div className="min-w-0 space-y-0.5">
                  <StepperLabel state={labelState}>
                    <span className="sm:hidden">{step.shortLabel}</span>
                    <span className="hidden sm:inline">{step.title}</span>
                  </StepperLabel>
                  <StepperDescription className="hidden sm:block">
                    {step.description}
                  </StepperDescription>
                </div>
              </div>
              {index < steps.length - 1 ? (
                <StepperConnector
                  state={connectorState}
                  className="mt-[1.1rem] ml-2 sm:ml-3"
                  aria-hidden="true"
                />
              ) : null}
            </StepperItem>
          );
        })}
      </StepperList>
    </Stepper>
  );
}
