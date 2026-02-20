import { useEffect, useMemo, useRef, useState } from "react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { OTP_LENGTH, validateOtpDigits } from "@/lib/validators/auth";

const TEN_MINUTES_IN_SECONDS = 10 * 60;

type VerificationCodeFormProps = {
  email: string;
  onComplete: (code: string) => void;
  onBack: () => void;
};

export function VerificationCodeForm({
  email,
  onComplete,
  onBack,
}: VerificationCodeFormProps) {
  const [digits, setDigits] = useState<string[]>(Array(OTP_LENGTH).fill(""));
  const [secondsLeft, setSecondsLeft] = useState(TEN_MINUTES_IN_SECONDS);
  const [error, setError] = useState<string | null>(null);
  const inputRefs = useRef<Array<HTMLInputElement | null>>([]);

  useEffect(() => {
    if (secondsLeft <= 0) {
      return;
    }

    const timer = setInterval(() => {
      setSecondsLeft((prev) => (prev <= 1 ? 0 : prev - 1));
    }, 1000);

    return () => clearInterval(timer);
  }, [secondsLeft]);

  const timeDisplay = useMemo(() => {
    const minutes = Math.floor(secondsLeft / 60)
      .toString()
      .padStart(2, "0");
    const seconds = (secondsLeft % 60).toString().padStart(2, "0");
    return `${minutes}:${seconds}`;
  }, [secondsLeft]);

  const submitIfComplete = (nextDigits: string[]) => {
    if (nextDigits.some((digit) => digit === "")) {
      return;
    }

    const validationError = validateOtpDigits(nextDigits);
    if (validationError) {
      setError(validationError);
      return;
    }

    setError(null);
    onComplete(nextDigits.join(""));
  };

  const handleDigitChange = (index: number, rawValue: string) => {
    if (secondsLeft <= 0) {
      return;
    }

    const value = rawValue.replace(/\D/g, "").slice(-1);
    const nextDigits = [...digits];
    nextDigits[index] = value;
    setDigits(nextDigits);
    setError(null);

    if (value && index < OTP_LENGTH - 1) {
      inputRefs.current[index + 1]?.focus();
    }

    submitIfComplete(nextDigits);
  };

  const handleKeyDown = (
    index: number,
    event: React.KeyboardEvent<HTMLInputElement>,
  ) => {
    if (event.key === "Backspace" && digits[index] === "" && index > 0) {
      inputRefs.current[index - 1]?.focus();
    }
  };

  const handlePaste = (event: React.ClipboardEvent<HTMLInputElement>) => {
    event.preventDefault();
    if (secondsLeft <= 0) {
      return;
    }

    const pasted = event.clipboardData
      .getData("text")
      .replace(/\D/g, "")
      .slice(0, OTP_LENGTH);

    if (!pasted) {
      return;
    }

    const nextDigits = Array(OTP_LENGTH)
      .fill("")
      .map((_, index) => pasted[index] ?? "");

    setDigits(nextDigits);
    setError(null);

    const focusIndex = Math.min(pasted.length, OTP_LENGTH - 1);
    inputRefs.current[focusIndex]?.focus();
    submitIfComplete(nextDigits);
  };

  const handleManualSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const validationError = validateOtpDigits(digits);
    if (validationError) {
      setError(validationError);
      return;
    }

    setError(null);
    onComplete(digits.join(""));
  };

  return (
    <Card className="w-full max-w-xl rounded-2xl border-border/80 shadow-lg animate-in fade-in-0 zoom-in-95 duration-500">
      <CardHeader className="space-y-2 text-center">
        <CardTitle className="text-2xl">Verify your email</CardTitle>
        <p className="text-sm text-muted-foreground">
          Enter the 6-digit code sent to {email}.
        </p>
        <p className="text-sm font-medium text-foreground">
          Code expires in {timeDisplay}
        </p>
      </CardHeader>
      <CardContent>
        <form className="space-y-5" onSubmit={handleManualSubmit} noValidate>
          <div className="flex justify-center gap-2 sm:gap-3">
            {digits.map((digit, index) => (
              <input
                key={index}
                ref={(element) => {
                  inputRefs.current[index] = element;
                }}
                type="text"
                inputMode="numeric"
                pattern="[0-9]*"
                maxLength={1}
                value={digit}
                onChange={(event) => handleDigitChange(index, event.target.value)}
                onKeyDown={(event) => handleKeyDown(index, event)}
                onPaste={handlePaste}
                disabled={secondsLeft <= 0}
                className="h-12 w-12 rounded-full border border-border bg-background text-center text-lg font-semibold outline-none transition-colors focus:border-ring focus:ring-2 focus:ring-ring/40 disabled:opacity-60 sm:h-14 sm:w-14"
                aria-label={`Verification digit ${index + 1}`}
              />
            ))}
          </div>
          {error ? (
            <p className="text-center text-sm text-destructive" role="alert">
              {error}
            </p>
          ) : null}
          {secondsLeft === 0 ? (
            <p className="text-center text-sm text-destructive">
              The code expired. Go back and request a new one.
            </p>
          ) : null}
          <div className="flex items-center justify-center gap-3">
            <Button type="button" variant="outline" onClick={onBack}>
              Back
            </Button>
            <Button type="submit" disabled={secondsLeft <= 0}>
              Verify code
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
