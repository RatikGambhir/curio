import type { ReactElement } from "react";
import {
  BrowserRouter as Router,
  Navigate,
  Route,
  Routes,
  useLocation,
} from "react-router-dom";

import "./App.css";
import Atlas from "./pages/Atlas";
import Chat from "./pages/Chat";
import DesktopConstruction from "./pages/DesktopConstruction";
import Home from "./pages/Home";
import Landing from "./pages/Landing";
import Login from "./pages/Login";
import ProfileSetupWizard from "./pages/ProfileSetupWizard";
import ProfileSettings from "./pages/ProfileSettings";
import Vault from "./pages/Vault";
import { useAuthenticatedUser } from "./hooks/useAuthenticatedUser";

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

function App() {
  return (
    <Router>
      <Routes>
        <Route
          path="/login"
          element={
            <RedirectIfAuthenticated>
              <Login />
            </RedirectIfAuthenticated>
          }
        />
        <Route path="/verify-email" element={<Navigate replace to="/login" />} />
        <Route
          path="/profile-setup"
          element={
            <RequireAuth>
              <ProfileSetupWizard />
            </RequireAuth>
          }
        />
        <Route path="/desktop-construction" element={<DesktopConstruction />} />
        <Route path="/" element={<Landing />} />
        <Route path="/chat" element={<RequireAuth><Chat /></RequireAuth>} />
        <Route path="/atlas" element={<RequireAuth><Atlas /></RequireAuth>} />
        <Route path="/home" element={<RequireAuth><Home /></RequireAuth>} />
        <Route path="/vault" element={<RequireAuth><Vault /></RequireAuth>} />
        <Route path="/profile" element={<RequireAuth><ProfileSettings /></RequireAuth>} />
        <Route path="/settings" element={<RequireAuth><ProfileSettings /></RequireAuth>} />
      </Routes>
    </Router>
  );
}

export default App;
