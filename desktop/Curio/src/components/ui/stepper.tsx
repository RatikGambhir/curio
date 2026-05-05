import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "@/lib/utils";

const stepperIndicatorVariants = cva(
  "flex h-9 w-9 items-center justify-center rounded-full border text-sm font-semibold transition-colors",
  {
    variants: {
      state: {
        completed: "border-primary bg-primary text-primary-foreground",
        active: "border-primary bg-background text-primary",
        upcoming: "border-border bg-background text-muted-foreground",
      },
    },
    defaultVariants: {
      state: "upcoming",
    },
  },
);

const stepperConnectorVariants = cva("h-px flex-1 border-t", {
  variants: {
    state: {
      completed: "border-primary",
      upcoming: "border-border",
    },
  },
  defaultVariants: {
    state: "upcoming",
  },
});

const stepperLabelVariants = cva("text-xs font-semibold", {
  variants: {
    state: {
      current: "text-foreground",
      completed: "text-foreground",
      upcoming: "text-muted-foreground",
    },
  },
  defaultVariants: {
    state: "upcoming",
  },
});

function Stepper({ className, ...props }: React.ComponentProps<"nav">) {
  return (
    <nav
      data-slot="stepper"
      className={cn("w-full", className)}
      {...props}
    />
  );
}

function StepperList({ className, ...props }: React.ComponentProps<"ol">) {
  return (
    <ol
      data-slot="stepper-list"
      className={cn("flex w-full items-start gap-2 sm:gap-3", className)}
      {...props}
    />
  );
}

function StepperItem({ className, ...props }: React.ComponentProps<"li">) {
  return (
    <li
      data-slot="stepper-item"
      className={cn("flex min-w-0 flex-1 items-start", className)}
      {...props}
    />
  );
}

function StepperIndicator({
  className,
  state,
  ...props
}: React.ComponentProps<"span"> &
  VariantProps<typeof stepperIndicatorVariants>) {
  return (
    <span
      data-slot="stepper-indicator"
      className={cn(stepperIndicatorVariants({ state }), className)}
      {...props}
    />
  );
}

function StepperConnector({
  className,
  state,
  ...props
}: React.ComponentProps<"div"> &
  VariantProps<typeof stepperConnectorVariants>) {
  return (
    <div
      data-slot="stepper-connector"
      className={cn(stepperConnectorVariants({ state }), className)}
      {...props}
    />
  );
}

function StepperLabel({
  className,
  state,
  ...props
}: React.ComponentProps<"p"> &
  VariantProps<typeof stepperLabelVariants>) {
  return (
    <p
      data-slot="stepper-label"
      className={cn(stepperLabelVariants({ state }), className)}
      {...props}
    />
  );
}

function StepperDescription({ className, ...props }: React.ComponentProps<"p">) {
  return (
    <p
      data-slot="stepper-description"
      className={cn("text-[11px] text-muted-foreground", className)}
      {...props}
    />
  );
}

export {
  Stepper,
  StepperConnector,
  StepperDescription,
  StepperIndicator,
  StepperItem,
  StepperLabel,
  StepperList,
};
