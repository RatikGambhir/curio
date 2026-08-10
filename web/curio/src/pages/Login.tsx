import { useState } from "react";
import { useNavigate } from "react-router-dom";

import { EmailLoginForm } from "@/components/auth/email-login-form";
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser";
import { validateEmail } from "@/lib/validators/auth";
import curioLogo from "../assets/curio-logo.png";

function Login() {
  const navigate = useNavigate();
  const { loginUser } = useAuthenticatedUser();
  const [email, setEmail] = useState("");
  const [emailError, setEmailError] = useState<string | null>(null);

  const handleEmailSubmit = () => {
    const trimmedEmail = email.trim().toLowerCase();
    const validationError = validateEmail(trimmedEmail);
    if (validationError) {
      setEmailError(validationError);
      return;
    }

    setEmailError(null);
    loginUser(trimmedEmail);
    navigate("/home", { replace: true });
  };

  return (
    <div className="flex h-screen flex-col items-center justify-center gap-8 bg-muted/30 px-4">
      <img
        src={curioLogo}
        alt="Curio Logo"
        className="mb-8 w-64 h-auto"
        style={{ mixBlendMode: "multiply" }}
      />
      <EmailLoginForm
        email={email}
        error={emailError}
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

export default Login;
