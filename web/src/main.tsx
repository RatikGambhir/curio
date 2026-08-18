import { createRoot } from "react-dom/client";
import "@fontsource-variable/inter/opsz.css";
import "@fontsource-variable/inter/opsz-italic.css";
import "./index.css";
import App from "./App.tsx";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { platformServices } from "@curio/platform-runtime";
import { AuthProvider } from "./lib/providers/auth-provider.tsx";
import { PlatformProvider } from "./platform/provider.tsx";

const queryClient = new QueryClient();

createRoot(document.getElementById("root")!).render(
  <PlatformProvider services={platformServices}>
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <App />
      </AuthProvider>
    </QueryClientProvider>
  </PlatformProvider>,
);
