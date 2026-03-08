import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";

type AuthUser = {
  id: string;
  name: string;
  email: string;
  avatar: string;
};

type AuthContextValue = {
  user: AuthUser | null;
  isAuthenticated: boolean;
  pendingEmail: string;
  setPendingEmail: (email: string) => void;
  completeLogin: (email: string) => void;
  logoutUser: () => void;
};

const AUTH_STORAGE_KEY = "curio-desktop-auth-user";
const PENDING_EMAIL_KEY = "curio-desktop-pending-email";

const AuthContext = createContext<AuthContextValue | null>(null);

function buildMockUser(email: string): AuthUser {
  const localPart = email.split("@")[0] ?? "curio";
  const normalizedName = localPart
    .split(/[._-]/g)
    .filter(Boolean)
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(" ");

  return {
    id: `desktop-${email}`,
    name: normalizedName || "Curio User",
    email,
    avatar: "",
  };
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<AuthUser | null>(null);
  const [pendingEmail, setPendingEmailState] = useState("");

  useEffect(() => {
    const storedUser = window.localStorage.getItem(AUTH_STORAGE_KEY);
    const storedPendingEmail = window.localStorage.getItem(PENDING_EMAIL_KEY);

    if (storedUser) {
      setUser(JSON.parse(storedUser) as AuthUser);
    }
    if (storedPendingEmail) {
      setPendingEmailState(storedPendingEmail);
    }
  }, []);

  const setPendingEmail = (email: string) => {
    const normalizedEmail = email.trim().toLowerCase();
    setPendingEmailState(normalizedEmail);
    if (normalizedEmail) {
      window.localStorage.setItem(PENDING_EMAIL_KEY, normalizedEmail);
      return;
    }
    window.localStorage.removeItem(PENDING_EMAIL_KEY);
  };

  const completeLogin = (email: string) => {
    const normalizedEmail = email.trim().toLowerCase();
    const nextUser = buildMockUser(normalizedEmail);
    setUser(nextUser);
    setPendingEmailState(normalizedEmail);
    window.localStorage.setItem(AUTH_STORAGE_KEY, JSON.stringify(nextUser));
    window.localStorage.setItem(PENDING_EMAIL_KEY, normalizedEmail);
  };

  const logoutUser = () => {
    setUser(null);
    window.localStorage.removeItem(AUTH_STORAGE_KEY);
  };

  const value = useMemo<AuthContextValue>(
    () => ({
      user,
      isAuthenticated: Boolean(user),
      pendingEmail,
      setPendingEmail,
      completeLogin,
      logoutUser,
    }),
    [pendingEmail, user],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error("useAuth must be used within an AuthProvider.");
  }
  return context;
}
