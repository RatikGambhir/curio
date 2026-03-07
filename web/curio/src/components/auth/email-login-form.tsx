import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
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
    <Card className="w-full max-w-md rounded-2xl border-border/80 shadow-md">
      <CardHeader className="space-y-2 text-center">
        <CardTitle className="text-2xl">Login to Curio</CardTitle>
        <p className="text-sm text-muted-foreground">
          Enter your email to receive a verification code.
        </p>
      </CardHeader>
      <CardContent>
        <form className="space-y-4" onSubmit={handleSubmit} noValidate>
          <div className="space-y-2">
            <Label htmlFor="email">Email</Label>
            <Input
              id="email"
              type="email"
              inputMode="email"
              autoComplete="email"
              placeholder="name@example.com"
              value={email}
              onChange={(event) => {
                onEmailChange(event.target.value);
              }}
              aria-invalid={Boolean(error)}
              className="h-11 rounded-xl"
            />
            {error ? (
              <p className="text-sm text-destructive" role="alert">
                {error}
              </p>
            ) : null}
          </div>
          <Button
            type="submit"
            className="h-11 w-full rounded-xl"
            disabled={isSubmitting}
          >
            {isSubmitting ? "Sending code..." : "Continue"}
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}
