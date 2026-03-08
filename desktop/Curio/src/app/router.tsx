import type { ReactElement } from "react";
import {
  HashRouter,
  Navigate,
  Route,
  Routes,
  useLocation,
} from "react-router-dom";

import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser";
import ChatPage from "@/pages/chat";
import HomePage from "@/pages/home";
import LoginPage from "@/pages/login";
import ProfilePage from "@/pages/profile";
import VerifyEmailPage from "@/pages/verify-email";

function RequireAuth({ children }: { children: ReactElement }) {
  const { isAuthenticated } = useAuthenticatedUser();
  const location = useLocation();

  if (!isAuthenticated) {
    return <Navigate replace to="/login" state={{ from: location.pathname }} />;
  }

  return children;
}

function RedirectIfAuthenticated({ children }: { children: ReactElement }) {
  const { isAuthenticated } = useAuthenticatedUser();
  return isAuthenticated ? <Navigate replace to="/home" /> : children;
}

export function AppRouter() {
  return (
    <HashRouter>
      <Routes>
        <Route
          path="/login"
          element={
            <RedirectIfAuthenticated>
              <LoginPage />
            </RedirectIfAuthenticated>
          }
        />
        <Route
          path="/verify-email"
          element={
            <RedirectIfAuthenticated>
              <VerifyEmailPage />
            </RedirectIfAuthenticated>
          }
        />
        <Route
          path="/home"
          element={
            <RequireAuth>
              <HomePage />
            </RequireAuth>
          }
        />
        <Route
          path="/chat"
          element={
            <RequireAuth>
              <ChatPage />
            </RequireAuth>
          }
        />
        <Route
          path="/profile"
          element={
            <RequireAuth>
              <ProfilePage />
            </RequireAuth>
          }
        />
        <Route path="*" element={<Navigate replace to="/login" />} />
      </Routes>
    </HashRouter>
  );
}
