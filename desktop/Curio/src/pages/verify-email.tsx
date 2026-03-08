import { useEffect, useMemo, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";

import curioLogo from "@/assets/curio-logo.png";
import { VerificationCodeForm } from "@/components/auth/verification-code-form";
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser";
import { validateEmail } from "@/lib/validators/auth";

function VerifyEmailPage() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const { completeLogin, pendingEmail } = useAuthenticatedUser();
  const [isVerifyingCode, setIsVerifyingCode] = useState(false);
  const email = searchParams.get("email") ?? pendingEmail ?? "";

  const emailError = useMemo(() => validateEmail(email), [email]);

  useEffect(() => {
    if (emailError) {
      navigate("/login", { replace: true });
    }
  }, [emailError, navigate]);

  if (emailError) {
    return null;
  }

  return (
    <div className="flex h-screen flex-col items-center justify-center gap-8 bg-muted/30 px-4">
      <img
        src={curioLogo}
        alt="Curio Logo"
        className="h-auto w-48"
        style={{ mixBlendMode: "multiply" }}
      />
      <VerificationCodeForm
        email={email}
        isSubmitting={isVerifyingCode}
        onBack={() => navigate("/login")}
        onComplete={(code) => {
          if (!code) {
            return;
          }

          setIsVerifyingCode(true);
          window.setTimeout(() => {
            completeLogin(email);
            setIsVerifyingCode(false);
            navigate("/home", { replace: true });
          }, 450);
        }}
      />
    </div>
  );
}

export default VerifyEmailPage;
