type RegisterData = {
  firstName: string;
  lastName: string;
  email: string;
  password: string;
  phone: string | null;
};

type AuthContextValue = {
  user: string | null; // you can type this as `User | null` later
  isAuthenticated: boolean;
  loginUser: (user: string | null) => void;
};

export type { RegisterData, AuthContextValue };
