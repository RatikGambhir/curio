import { CultivationIndex } from "@/components/home/cultivation-index";
import {
  RECENT_CULTIVATIONS,
  UPCOMING_ITEMS,
} from "@/components/home/home.mock-data";
import { QuickStart } from "@/components/home/quick-start";
import { ThoughtPad } from "@/components/home/thought-pad";
import { UpcomingAgenda } from "@/components/home/upcoming-agenda";
import { PageHeader } from "@/components/page-header";
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser";

const MASTHEAD_DATE = new Intl.DateTimeFormat(undefined, {
  weekday: "long",
  day: "numeric",
  month: "long",
});

function localIsoDate(date: Date) {
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${date.getFullYear()}-${month}-${day}`;
}

function greetingForHour(hour: number) {
  if (hour < 12) return "Good morning";
  if (hour < 18) return "Good afternoon";
  return "Good evening";
}

const Home = () => {
  const { user } = useAuthenticatedUser();
  const firstName = user?.name.trim().split(/\s+/)[0] || "there";
  const now = new Date();

  return (
    <>
      <PageHeader title="Home" titleIsHeading={false} />

      <div className="min-h-0 min-w-0 flex-1 overflow-y-auto overscroll-contain">
        <div className="mx-auto w-full max-w-[72rem] px-5 pb-16 pt-10 sm:px-8 lg:px-12 lg:pt-14">
          <header className="rise-in max-w-3xl">
            <p className="eyebrow text-muted-foreground">
              <time dateTime={localIsoDate(now)}>
                {MASTHEAD_DATE.format(now)}
              </time>
            </p>
            <h1 className="mt-3 font-display text-display-lg text-foreground">
              {greetingForHour(now.getHours())}, {firstName}.
            </h1>
            <p className="mt-3 font-display text-xl italic leading-snug text-muted-foreground sm:text-[1.375rem]">
              The garden is looking lush today. What are we cultivating?
            </p>
          </header>

          <div className="mt-10 grid min-w-0 gap-x-12 gap-y-10 lg:mt-12 lg:grid-cols-[minmax(0,1fr)_17.5rem]">
            <div className="rise-in min-w-0 [--rise-index:1]">
              <ThoughtPad userId={user?.id ?? "anonymous"} />
            </div>
            <div className="rise-in grid min-w-0 content-start gap-10 [--rise-index:2] sm:grid-cols-2 lg:grid-cols-1">
              <UpcomingAgenda items={UPCOMING_ITEMS} />
              <QuickStart />
            </div>
          </div>

          <div className="rise-in mt-14 [--rise-index:3] lg:mt-16">
            <CultivationIndex items={RECENT_CULTIVATIONS} />
          </div>
        </div>
      </div>
    </>
  );
};

export default Home;
