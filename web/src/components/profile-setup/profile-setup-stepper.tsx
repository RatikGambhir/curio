import { Check } from "lucide-react";

import { cn } from "@/lib/utils";

import type {
  ProfileSetupStep,
  ProfileSetupStepMeta,
} from "./profile-setup.types";

type ProfileSetupStepperProps = {
  currentStep: ProfileSetupStep;
  steps: readonly ProfileSetupStepMeta[];
};

/* The full step list, set on the ink page beside the form on wide screens. */
export function ProfileSetupStepper({
  currentStep,
  steps,
}: ProfileSetupStepperProps) {
  return (
    <nav aria-label="Profile setup progress">
      <ol className="space-y-1">
        {steps.map((step) => {
          const isCompleted = step.id < currentStep;
          const isActive = step.id === currentStep;

          return (
            <li
              key={step.id}
              aria-current={isActive ? "step" : undefined}
              className="flex items-start gap-4 py-2"
            >
              <span
                className={cn(
                  "mt-0.5 flex size-6 shrink-0 items-center justify-center rounded-full border font-mono text-2xs transition-colors",
                  isCompleted &&
                    "border-sidebar-primary bg-sidebar-primary text-sidebar-primary-foreground",
                  isActive && "border-sidebar-foreground text-sidebar-foreground",
                  !isCompleted &&
                    !isActive &&
                    "border-sidebar-border text-sidebar-muted-foreground",
                )}
              >
                {isCompleted ? (
                  <Check className="size-3.5" aria-label="Completed" />
                ) : (
                  step.id
                )}
              </span>
              <span className="min-w-0">
                <span
                  className={cn(
                    "block text-sm font-medium",
                    isActive || isCompleted
                      ? "text-sidebar-foreground"
                      : "text-sidebar-muted-foreground",
                  )}
                >
                  {step.title}
                </span>
                <span className="mt-0.5 block text-xs text-sidebar-muted-foreground">
                  {step.description}
                </span>
              </span>
            </li>
          );
        })}
      </ol>
    </nav>
  );
}

/* A compact bar for narrow screens, where the ink page is hidden. */
export function ProfileSetupProgress({
  currentStep,
  steps,
}: ProfileSetupStepperProps) {
  return (
    <div className="lg:hidden" aria-hidden="true">
      <div className="flex gap-1">
        {steps.map((step) => (
          <span
            key={step.id}
            className={cn(
              "h-1 flex-1 rounded-full transition-colors",
              step.id <= currentStep ? "bg-primary" : "bg-secondary",
            )}
          />
        ))}
      </div>
    </div>
  );
}
