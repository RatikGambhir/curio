import { Link } from "react-router-dom";

import { AppSidebar } from "@/components/app-sidebar";
import { CultivationCard } from "@/components/home/cultivation-card";
import {
  RECENT_CULTIVATIONS,
  UPCOMING_ITEMS,
} from "@/components/home/home.mock-data";
import { NewSeedCard } from "@/components/home/new-seed-card";
import { ThoughtCard } from "@/components/home/thought-card";
import { UpcomingCard } from "@/components/home/upcoming-card";
import { PageHeader } from "@/components/page-header";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser";

function greetingForHour(hour: number) {
  if (hour < 12) return "Morning";
  if (hour < 18) return "Afternoon";
  return "Evening";
}

const Home = () => {
  const { user } = useAuthenticatedUser();
  const firstName = user?.name.trim().split(/\s+/)[0] || "there";
  const greeting = greetingForHour(new Date().getHours());

  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <div className="flex h-full w-full min-w-0 flex-col">
          <PageHeader />

          <div className="min-h-0 min-w-0 flex-1 overflow-y-auto overscroll-contain">
            <div className="mx-auto w-full max-w-[70rem] px-4 py-8 sm:px-6 sm:py-10 lg:px-8">
              <header>
                <h1 className="text-[clamp(2.25rem,3.8vw,3rem)] font-semibold leading-[1.05] tracking-[-0.025em] text-foreground">
                  {greeting}, {firstName}
                </h1>
                <p className="mt-2 text-base leading-7 text-muted-foreground sm:text-[1.0625rem]">
                  The garden is looking lush today. What are we cultivating?
                </p>
              </header>

              <div className="mt-8 grid min-w-0 gap-5 lg:grid-cols-[minmax(0,2.1fr)_minmax(17rem,1fr)]">
                <ThoughtCard userId={user?.id ?? "anonymous"} />
                <UpcomingCard items={UPCOMING_ITEMS} />
              </div>

              <section
                aria-labelledby="recent-cultivations-title"
                className="mt-9"
              >
                <div className="flex items-end justify-between gap-4">
                  <h2
                    id="recent-cultivations-title"
                    className="text-[1.65rem] font-semibold leading-tight tracking-[-0.015em] text-foreground"
                  >
                    Recent Cultivations
                  </h2>
                  <Link
                    to="/vault"
                    className="shrink-0 rounded-md px-1.5 py-1 text-sm font-semibold text-muted-foreground outline-none transition-colors hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
                  >
                    View All
                  </Link>
                </div>

                <ul className="mt-6 grid min-w-0 grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                  {RECENT_CULTIVATIONS.map((cultivation) => (
                    <li key={cultivation.title} className="min-w-0">
                      <CultivationCard cultivation={cultivation} />
                    </li>
                  ))}
                  <li className="min-w-0">
                    <NewSeedCard />
                  </li>
                </ul>
              </section>
            </div>
          </div>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
};

export default Home;
