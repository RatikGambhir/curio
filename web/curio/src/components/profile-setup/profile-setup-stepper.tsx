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

export function ProfileSetupStepper({
  currentStep,
  steps,
}: ProfileSetupStepperProps) {
  return (
    <nav aria-label="Profile setup progress" className="w-full">
      <ol className="flex w-full items-start gap-2 sm:gap-3">
        {steps.map((step, index) => {
          const isCompleted = step.id < currentStep;
          const isActive = step.id === currentStep;
          const isConnectorComplete = currentStep > step.id;

          return (
            <li key={step.id} className="flex min-w-0 flex-1 items-start">
              <div className="flex min-w-0 flex-col items-center gap-2 text-center">
                <span
                  className={cn(
                    "flex h-9 w-9 items-center justify-center rounded-full border text-sm font-semibold transition-colors",
                    isCompleted && "border-primary bg-primary text-primary-foreground",
                    isActive && "border-primary bg-background text-primary",
                    !isCompleted &&
                      !isActive &&
                      "border-border bg-background text-muted-foreground",
                  )}
                >
                  {isCompleted ? <Check className="size-4" /> : step.id}
                </span>
                <div className="min-w-0 space-y-0.5">
                  <p
                    className={cn(
                      "text-xs font-semibold",
                      isActive || isCompleted
                        ? "text-foreground"
                        : "text-muted-foreground",
                    )}
                  >
                    <span className="sm:hidden">{step.shortLabel}</span>
                    <span className="hidden sm:inline">{step.title}</span>
                  </p>
                  <p className="hidden text-[11px] text-muted-foreground sm:block">
                    {step.description}
                  </p>
                </div>
              </div>
              {index < steps.length - 1 ? (
                <div
                  className={cn(
                    "mt-[1.1rem] ml-2 h-px flex-1 border-t sm:ml-3",
                    isConnectorComplete ? "border-primary" : "border-border",
                  )}
                  aria-hidden="true"
                />
              ) : null}
            </li>
          );
        })}
      </ol>
    </nav>
  );
}
