import { useState } from "react";
import { useNavigate } from "react-router-dom";

import { AuthLayout } from "@/components/auth/auth-layout";
import { EmailLoginForm } from "@/components/auth/email-login-form";
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser";
import { validateEmail } from "@/lib/validators/auth";

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
    <AuthLayout statement="For every question, an answer worth keeping.">
      <p className="eyebrow text-muted-foreground">Sign in</p>
      <h1 className="mt-3 font-display text-display-md text-foreground">
        Welcome back.
      </h1>
      <p className="mt-3 text-sm leading-relaxed text-muted-foreground">
        Enter your email to open your workspace. This build uses a local
        development session, so no password is needed.
      </p>
      <div className="mt-8">
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
    </AuthLayout>
  );
}

export default Login;
