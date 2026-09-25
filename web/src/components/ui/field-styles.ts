import { cn } from "@/lib/utils"

/* Shared by Input and Textarea so every text field in the app has the same
   border, fill, focus and invalid treatment. */
export const fieldClassName = cn(
  "w-full min-w-0 rounded-md border border-input bg-card text-base text-foreground shadow-2xs transition-[border-color,box-shadow] outline-none md:text-sm",
  "placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground",
  "hover:border-muted-foreground/40",
  "focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/20",
  "aria-invalid:border-destructive aria-invalid:ring-destructive/15",
  "disabled:cursor-not-allowed disabled:bg-secondary disabled:opacity-60",
)
