import { useNavigate } from "react-router-dom";

import { EmailLoginForm } from "@/components/auth/email-login-form";
import curioLogo from "../assets/curio-logo.png";

function Login() {
  const navigate = useNavigate();

  return (
    <div className="flex h-screen flex-col items-center justify-center gap-8 bg-muted/30 px-4">
      <img
        src={curioLogo}
        alt="Curio Logo"
        className="mb-8 w-64 h-auto"
        style={{ mixBlendMode: "multiply" }}
      />
      <EmailLoginForm
        onContinue={(email) =>
          navigate(`/verify-email?email=${encodeURIComponent(email)}`)
        }
      />
    </div>
  );
}

export default Login;
