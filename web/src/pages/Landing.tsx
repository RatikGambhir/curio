import { useNavigate } from "react-router-dom";

import {
  ContactSection,
  HeroSection,
  HowItWorksSection,
  LandingFooter,
  LandingHeader,
  SurfacesSection,
} from "@/components/landing/landing-sections";

function scrollToSection(id: string) {
  const prefersReducedMotion = window.matchMedia(
    "(prefers-reduced-motion: reduce)",
  ).matches;
  document.getElementById(id)?.scrollIntoView({
    behavior: prefersReducedMotion ? "auto" : "smooth",
    block: "start",
  });
}

const Landing = () => {
  const navigate = useNavigate();

  return (
    <div className="min-h-svh bg-background text-foreground">
      <LandingHeader
        onContactClick={() => scrollToSection("contact")}
        onWebClick={() => navigate("/login")}
        onDesktopClick={() => navigate("/login")}
      />
      <main>
        <HeroSection onHowItWorks={() => scrollToSection("how-it-works")} />
        <HowItWorksSection />
        <SurfacesSection />
        <ContactSection />
      </main>
      <LandingFooter />
    </div>
  );
};

export default Landing;
