import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

type EmailLoginFormProps = {
  email: string;
  error: string | null;
  isSubmitting?: boolean;
  onEmailChange: (email: string) => void;
  onSubmit: () => void;
};

export function EmailLoginForm({
  email,
  error,
  isSubmitting = false,
  onEmailChange,
  onSubmit,
}: EmailLoginFormProps) {
  const handleSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    onSubmit();
  };

  return (
    <form className="space-y-5" onSubmit={handleSubmit} noValidate>
      <div className="space-y-2">
        <Label htmlFor="email">Email</Label>
        <Input
          id="email"
          type="email"
          inputMode="email"
          autoComplete="email"
          autoFocus
          placeholder="name@example.com"
          value={email}
          onChange={(event) => {
            onEmailChange(event.target.value);
          }}
          aria-invalid={Boolean(error)}
          aria-describedby={error ? "email-error" : undefined}
          className="h-11"
        />
        {error ? (
          <p id="email-error" className="text-sm text-destructive" role="alert">
            {error}
          </p>
        ) : null}
      </div>
      <Button
        type="submit"
        size="lg"
        className="w-full"
        disabled={isSubmitting}
      >
        {isSubmitting ? "Opening Curio…" : "Continue"}
      </Button>
    </form>
  );
}
