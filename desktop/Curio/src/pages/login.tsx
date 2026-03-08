import { useState } from "react";
import { useNavigate } from "react-router-dom";

import curioLogo from "@/assets/curio-logo.png";
import { EmailLoginForm } from "@/components/auth/email-login-form";
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser";
import { validateEmail } from "@/lib/validators/auth";

function LoginPage() {
  const navigate = useNavigate();
  const { setPendingEmail } = useAuthenticatedUser();
  const [email, setEmail] = useState("");
  const [emailError, setEmailError] = useState<string | null>(null);
  const [isSendingCode, setIsSendingCode] = useState(false);

  const handleEmailSubmit = async () => {
    const trimmedEmail = email.trim().toLowerCase();
    const validationError = validateEmail(trimmedEmail);

    if (validationError) {
      setEmailError(validationError);
      return;
    }

    setEmailError(null);
    setIsSendingCode(true);

    window.setTimeout(() => {
      setPendingEmail(trimmedEmail);
      setIsSendingCode(false);
      navigate(`/verify-email?email=${encodeURIComponent(trimmedEmail)}`);
    }, 450);
  };

  return (
    <div className="flex h-screen flex-col items-center justify-center gap-8 bg-muted/30 px-4">
      <img
        src={curioLogo}
        alt="Curio Logo"
        className="mb-8 h-auto w-64"
        style={{ mixBlendMode: "multiply" }}
      />
      <EmailLoginForm
        email={email}
        error={emailError}
        isSubmitting={isSendingCode}
        onEmailChange={(nextEmail) => {
          setEmail(nextEmail);
          if (emailError) {
            setEmailError(null);
          }
        }}
        onSubmit={handleEmailSubmit}
      />
    </div>
  );
}

export default LoginPage;
