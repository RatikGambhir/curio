import { FeatureStepper } from "@/components/feature-stepper";
import { FeatureBentoGrid } from "@/components/feature-bento-grid";
import {
  ContactSection,
  HeroSection,
  LandingFooter,
  LandingNavbar,
} from "@/components/landing/landing-sections";

const featureShowcaseItems = [
  {
    image:
      "https://images.unsplash.com/photo-1515879218367-8466d910aaa4?auto=format&fit=crop&w=1600&q=80",
    title: "Ask Curio Anything",
    description:
      "Get instant answers and keep each response as part of your growing, searchable knowledge history.",
  },
  {
    image:
      "https://images.unsplash.com/photo-1456324504439-367cee3b3c32?auto=format&fit=crop&w=1600&q=80",
    title: "Find Insights Fast",
    description:
      "Search across every note, topic, and response in one place to quickly return to what matters.",
  },
  {
    image:
      "https://images.unsplash.com/photo-1501504905252-473c47e087f8?auto=format&fit=crop&w=1600&q=80",
    title: "Build Connected Notes",
    description:
      "Curio helps you connect ideas over time so your learning becomes more structured and reusable.",
  },
  {
    image:
      "https://images.unsplash.com/photo-1513258496099-48168024aec0?auto=format&fit=crop&w=1600&q=80",
    title: "Keep Learning Daily",
    description:
      "Turn everyday questions into long-term understanding with a vault that compounds as you use it.",
  },
];

const Landing = () => {
  const scrollToContact = () => {
    document.getElementById("contact")?.scrollIntoView({
      behavior: "smooth",
      block: "start",
    });
  };

  return (
    <main className="h-screen snap-y snap-mandatory overflow-y-auto scroll-smooth bg-muted text-foreground">
      <section className="snap-start">
        <div className="flex min-h-screen flex-col">
          <LandingNavbar onContactClick={scrollToContact} />
          <HeroSection className="min-h-[calc(100vh-88px)] py-0" />
        </div>
      </section>

      <section className="snap-start">
        <div className="flex min-h-screen items-center">
          <FeatureStepper
            heading="How Curio Works"
            features={featureShowcaseItems}
            className="pb-0"
          />
        </div>
      </section>

      <section className="snap-start">
        <div className="flex min-h-screen items-center">
          <FeatureBentoGrid className="pb-0" />
        </div>
      </section>

      <section className="snap-start">
        <div className="flex min-h-screen flex-col justify-center">
          <ContactSection className="pb-0" />
          <LandingFooter />
        </div>
      </section>
      {/*<BookSection page={currentPage} onNextPage={handleNextPage} />*/}
    </main>
  );
};

export default Landing;
