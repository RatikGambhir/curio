import { useState } from "react";
import { useNavigate } from "react-router-dom";

import { EmailLoginForm } from "@/components/auth/email-login-form";
import { VerificationCodeForm } from "@/components/auth/verification-code-form";
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser";
import { validateEmail } from "@/lib/validators/auth";
import { supabase } from "@/supabase";
import curioLogo from "../assets/curio-logo.png";

function Login() {
  const navigate = useNavigate();
  const { loginUser } = useAuthenticatedUser();
  const [step, setStep] = useState<"email" | "otp">("email");
  const [email, setEmail] = useState("");
  const [emailError, setEmailError] = useState<string | null>(null);
  const [otpError, setOtpError] = useState<string | null>(null);
  const [isSendingCode, setIsSendingCode] = useState(false);
  const [isVerifyingCode, setIsVerifyingCode] = useState(false);
  const handleEmailSubmit = async () => {
    const trimmedEmail = email.trim().toLowerCase();
    const validationError = validateEmail(trimmedEmail);
    if (validationError) {
      setEmailError(validationError);
      return;
    }

    setEmailError(null);
    setOtpError(null);
    setIsSendingCode(true);

    const { error } = await supabase.auth.signInWithOtp({
      email: trimmedEmail,
      options: {
        shouldCreateUser: true,
      },
    });

    setIsSendingCode(false);

    if (error) {
      const normalizedMessage =
        error.message.includes("email") && error.message.includes("not")
          ? "We couldn't send a code to that email right now. Please try again or create an account first."
          : error.message;
      setEmailError(normalizedMessage);
      return;
    }

    setEmail(trimmedEmail);
    setStep("otp");
  };

  const handleOtpComplete = async (code: string) => {
    setOtpError(null);
    setIsVerifyingCode(true);

    const { data, error } = await supabase.auth.verifyOtp({
      email,
      token: code,
      type: "email",
    });

    setIsVerifyingCode(false);

    if (error) {
      setOtpError(error.message);
      return;
    }

    loginUser(data.user?.id ?? null);
    navigate("/profile-setup");
  };

  return (
    <div className="flex h-screen flex-col items-center justify-center gap-8 bg-muted/30 px-4">
      <img
        src={curioLogo}
        alt="Curio Logo"
        className="mb-8 w-64 h-auto"
        style={{ mixBlendMode: "multiply" }}
      />
      {step === "email" ? (
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
      ) : (
        <VerificationCodeForm
          email={email}
          error={otpError ?? null}
          isSubmitting={isVerifyingCode}
          onBack={() => {
            setOtpError(null);
            setStep("email");
          }}
          onComplete={handleOtpComplete}
        />
      )}
    </div>
  );
}

export default Login;
