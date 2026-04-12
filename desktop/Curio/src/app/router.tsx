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
import ProfileSetupPage from "@/pages/profile-setup";
import SettingsPage from "@/pages/settings";
import VerifyEmailPage from "@/pages/verify-email";
import VaultPage from "@/pages/vault";

function RequireAuth({ children }: { children: ReactElement }) {
  const { isAuthenticated } = useAuthenticatedUser();
  const location = useLocation();

  if (!isAuthenticated) {
    return <Navigate replace to="/login" state={{ from: location.pathname }} />;
  }

  return children;
}

function RedirectIfAuthenticated({ children }: { children: ReactElement }) {
  return children;
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
          path="/profile-setup"
          element={
            <RequireAuth>
              <ProfileSetupPage />
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
          path="/vault"
          element={
            <RequireAuth>
              <VaultPage />
            </RequireAuth>
          }
        />
        <Route
          path="/profile"
          element={
            <RequireAuth>
              <SettingsPage />
            </RequireAuth>
          }
        />
        <Route
          path="/settings"
          element={
            <RequireAuth>
              <SettingsPage />
            </RequireAuth>
          }
        />
        <Route path="*" element={<Navigate replace to="/login" />} />
      </Routes>
    </HashRouter>
  );
}
