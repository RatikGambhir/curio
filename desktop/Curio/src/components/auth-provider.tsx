import {
  createContext,
  useCallback,
  useContext,
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
  loginUser: (email: string) => void;
  logoutUser: () => void;
};

const AUTH_STORAGE_KEY = "curio-desktop-mock-auth-user-v1";

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

function readStoredUser(): AuthUser | null {
  const storedUser = window.localStorage.getItem(AUTH_STORAGE_KEY);
  if (!storedUser) {
    return null;
  }

  try {
    const user = JSON.parse(storedUser) as Partial<AuthUser>;
    if (typeof user.email !== "string" || !user.email) {
      window.localStorage.removeItem(AUTH_STORAGE_KEY);
      return null;
    }
    return buildMockUser(user.email);
  } catch {
    window.localStorage.removeItem(AUTH_STORAGE_KEY);
    return null;
  }
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<AuthUser | null>(readStoredUser);

  const loginUser = useCallback((email: string) => {
    const normalizedEmail = email.trim().toLowerCase();
    const nextUser = buildMockUser(normalizedEmail);
    setUser(nextUser);
    window.localStorage.setItem(AUTH_STORAGE_KEY, JSON.stringify(nextUser));
  }, []);

  const logoutUser = useCallback(() => {
    setUser(null);
    window.localStorage.removeItem(AUTH_STORAGE_KEY);
  }, []);

  const value = useMemo<AuthContextValue>(
    () => ({
      user,
      isAuthenticated: Boolean(user),
      loginUser,
      logoutUser,
    }),
    [loginUser, logoutUser, user],
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
