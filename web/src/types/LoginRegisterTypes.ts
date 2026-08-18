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

export type { AuthContextValue, AuthUser };
