import { RegisterForm } from "@/components/register-form";
import curioLogo from "../assets/curio-logo.png";

const Register = () => {
  return (
    <div className="flex flex-col items-center justify-center h-screen">
      <img
        src={curioLogo}
        alt="Curio Logo"
        className="w-48 h-auto"
        style={{ mixBlendMode: "lighten" }}
      />
      <RegisterForm className="w-1/2" />
    </div>
  );
};

export default Register;
