import { useEffect, useMemo } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";

import { VerificationCodeForm } from "@/components/auth/verification-code-form";
import curioLogo from "@/assets/curio-logo.png";
import { validateEmail } from "@/lib/validators/auth";

function VerifyEmail() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const email = searchParams.get("email") ?? "";

  const emailError = useMemo(() => validateEmail(email), [email]);

  useEffect(() => {
    if (emailError) {
      navigate("/login", { replace: true });
    }
  }, [emailError, navigate]);

  if (emailError) return null;

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
        onBack={() => navigate("/login")}
        onComplete={() => navigate("/home")}
      />
    </div>
  );
}

export default VerifyEmail;
