import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser";

import { supabase } from "@/lib/SupabaseClient";

type LoginForm = {
  email: string;
  password: string;
};

export function LoginForm({
  className,
  ...props
}: React.ComponentProps<"div">) {
  const [loginInput, setLoginInput] = useState<LoginForm>({
    email: "",
    password: "",
  });

  const navigate = useNavigate();

  const handleLoginFormChange = (
    event: React.ChangeEvent<HTMLInputElement>,
  ) => {
    setLoginInput({
      ...loginInput,
      [event.target.id]: event.target.value,
    });
  };

  const { loginUser } = useAuthenticatedUser();

  const handleSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    console.log(loginInput);
    const { data, error } = await supabase.auth.signInWithPassword({
      email: loginInput.email,
      password: loginInput.password,
    });
    if (error) {
      //TODO: Handle error with invalid credentials error
      console.error(error);
    }
    if (data) {
      console.log(data);
      loginUser(data.user?.id ?? null);
      navigate("/home");
    }
  };

  return (
    <div className={cn("flex flex-col gap-6", className)} {...props}>
      <Card>
        <CardHeader className="text-center">
          <CardTitle className="text-xl">Welcome to Curio!</CardTitle>
        </CardHeader>

        <CardContent>
          <form
            onSubmit={(e) => {
              handleSubmit(e);
            }}
          >
            <div className="grid gap-6">
              <div className="flex flex-col gap-4">
                <Button variant="outline" className="w-full">
                  {/* TODO: Fix google logo coloring */}
                  <svg className="h-5 w-5 mr-3" viewBox="0 0 48 48">
                    <path
                      fill="#EA4335"
                      d="M24 9.5c3.54 0 6.04 1.53 7.43 2.82l5.52-5.52C33.42 3.19 28.97 1 24 1 14.82 1 7.18 6.96 4.44 15h6.96C12.31 11.33 17.63 9.5 24 9.5z"
                    />
                    <path
                      fill="#4285F4"
                      d="M46.1 24.5c0-1.53-.14-3.01-.41-4.43H24v8.39h12.4c-.53 2.81-2.17 5.19-4.61 6.79v5.64h7.43c4.36-4.02 6.88-9.93 6.88-16.39z"
                    />
                    <path
                      fill="#FBBC05"
                      d="M11.4 28.48A14.48 14.48 0 0 1 10 24c0-1.56.27-3.06.74-4.48H3.78A23.936 23.936 0 0 0 1 24c0 3.83.9 7.44 2.48 10.64l6.84-6.16z"
                    />
                    <path
                      fill="#34A853"
                      d="M24 47c6.48 0 11.92-2.15 15.89-5.85l-7.43-6.27C29.83 36.32 27.06 37.5 24 37.5c-6.37 0-11.69-3.83-13.6-9.5H3.44l-6.84 6.16C7.18 41.04 14.82 47 24 47z"
                    />
                  </svg>
                  Login with Google
                </Button>
              </div>
              <div className="after:border-border relative text-center text-sm after:absolute after:inset-0 after:top-1/2 after:z-0 after:flex after:items-center after:border-t">
                <span className="bg-card text-muted-foreground relative z-10 px-2">
                  Or continue with
                </span>
              </div>
              <div className="grid gap-6">
                <div className="grid gap-3">
                  <Label htmlFor="email">Email</Label>
                  <Input
                    id="email"
                    type="email"
                    placeholder="m@example.com"
                    required
                    onBlur={handleLoginFormChange}
                  />
                </div>
                <div className="grid gap-3">
                  <div className="flex items-center">
                    <Label htmlFor="password">Password</Label>
                    <a
                      href="#"
                      className="ml-auto text-sm underline-offset-4 hover:underline"
                    >
                      Forgot your password?
                    </a>
                  </div>
                  <Input
                    id="password"
                    type="password"
                    required
                    onBlur={handleLoginFormChange}
                  />
                </div>
                <Button type="submit" className="w-full">
                  Login
                </Button>
              </div>
              <div className="text-center text-sm">
                Don&apos;t have an account?{" "}
                <a href="/register" className="underline underline-offset-4">
                  Sign up
                </a>
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
      <div className="text-black *:[a]:hover:text-primary text-center text-xs text-balance *:[a]:underline *:[a]:underline-offset-4">
        By clicking continue, you have confirmed that you have read our{" "}
        <a href="#">Privacy Policy</a>.
      </div>
    </div>
  );
}
