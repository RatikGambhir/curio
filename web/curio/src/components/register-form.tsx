import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useState } from "react";
import { cn } from "@/lib/utils";
import type { RegisterData } from "@/types/LoginRegisterTypes";
import { supabase } from "@/lib/SupabaseClient";
import { useNavigate } from "react-router-dom";
export function RegisterForm({
  className,
  ...props
}: React.ComponentProps<"div">) {
  const navigate = useNavigate();
  const [registerEnabled, setRegisterEnabled] = useState(false);
  const [registerData, setRegisterData] = useState<RegisterData>({
    firstName: "",
    lastName: "",
    email: "",
    password: "",
    phone: null,
  });

  const handleFormChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setRegisterData({
      ...registerData,
      [e.target.id]: e.target.value,
    });
  };

  const isValidEmail = (email: string) => {
    return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
  };

  const registerUser = async () => {
    console.log("Registering user...", registerData);
    console.log("supasbase", supabase);
    // TODO: Add db trigger to supabase to write custom user data to custom table
    const { data, error } = await supabase.auth.signUp({
      phone: registerData.phone ?? "",
      email: registerData.email,
      password: registerData.password,
    });
    if (error) {
      console.error(error);
    } else {
      navigate("/home");
      console.log(data);
    }
  };

  const isValidPassword = (password: string) => {
    return (
      password.length >= 8 &&
      /[A-Z]/.test(password) &&
      /[a-z]/.test(password) &&
      /[0-9]/.test(password) &&
      /[!@#$%^&*]/.test(password)
    );
  };
  return (
    <div className={cn("flex flex-col gap-4", className)} {...props}>
      <Card>
        <CardHeader className="text-center">
          <CardTitle className="text-xl">Create an account</CardTitle>
        </CardHeader>
        <CardContent>
          <form
            onSubmit={(e) => {
              e.preventDefault();
              registerUser();
            }}
          >
            <div className="grid gap-6">
              <div className="grid gap-6">
                <div className="grid gap-3">
                  <Label htmlFor="firstName">First Name</Label>
                  <Input
                    id="firstName"
                    type="firstName"
                    name="firstName"
                    required
                    defaultValue={registerData.firstName}
                    onBlur={handleFormChange}
                  />
                </div>
                <div className="grid gap-3">
                  <Label htmlFor="lastName">Last Name</Label>
                  <Input
                    id="lastName"
                    type="lastName"
                    name="lastName"
                    required
                    defaultValue={registerData.lastName}
                    onBlur={handleFormChange}
                  />
                </div>
                <div className="grid gap-3">
                  <Label htmlFor="email">Email</Label>
                  <Input
                    id="email"
                    type="email"
                    name="email"
                    required
                    defaultValue={registerData.email}
                    onBlur={(e) => {
                      if (isValidEmail(e.target.value)) {
                        handleFormChange(e);
                      }
                    }}
                  />
                </div>
                <div className="grid gap-3">
                  <div className="flex items-center">
                    <Label htmlFor="password">Password</Label>
                  </div>
                  <Input
                    id="password"
                    type="password"
                    name="password"
                    required
                    defaultValue={registerData.password}
                    onBlur={(e) => {
                      if (isValidPassword(e.target.value)) {
                        handleFormChange(e);
                      }
                    }}
                  />
                </div>
                <div className="grid gap-3">
                  <div className="flex items-center">
                    <Label htmlFor="confirmPassword">Confirm Password</Label>
                  </div>
                  <Input
                    id="confirmPassword"
                    type="password"
                    name="confirmPassword"
                    required
                    onBlur={(e) => {
                      setRegisterEnabled(
                        e.target.value === registerData.password,
                      );
                    }}
                  />
                </div>
                <div className="grid gap-3">
                  <div className="flex items-center">
                    <Label htmlFor="phoneNumber">Phone Number</Label>
                  </div>
                  <Input
                    id="phoneNumber"
                    type="tel"
                    name="phoneNumber"
                    required
                    onBlur={(e) => {
                      //TODO: Handle phone number validation
                      handleFormChange(e);
                    }}
                  />
                </div>
                {
                  //TODO add loading state spinner
                  <Button
                    type="submit"
                    className="w-full"
                    disabled={!registerEnabled}
                  >
                    Register
                  </Button>
                }
              </div>
              <div className="text-center text-sm">
                Want to try it out first?{" "}
                <a href="/home" className="underline underline-offset-4">
                  Try the demo
                </a>
              </div>
            </div>
          </form>
        </CardContent>
      </Card>
      <div className="text-white *:[a]:hover:text-primary text-center text-xs text-balance *:[a]:underline *:[a]:underline-offset-4">
        By clicking register, you have confirmed that you have read our{" "}
        <a href="#">Privacy Policy</a>.
      </div>
    </div>
  );
}
