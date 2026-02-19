import { LoginForm } from "../components/login-form";
import curioLogo from "../assets/curio-logo.png";

function Login() {
  return (
    <div className="flex flex-col items-center justify-center h-screen ">
      <img
        src={curioLogo}
        alt="Curio Logo"
        className="mb-8 w-64 h-auto"
        style={{ mixBlendMode: "multiply" }}
      />
      <LoginForm />
    </div>
  );
}

export default Login;
