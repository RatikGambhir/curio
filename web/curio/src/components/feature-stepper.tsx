"use client";

import { AnimatePresence, motion } from "framer-motion";
import {
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  ChevronUp,
  PlusCircle,
} from "lucide-react";
import { useState } from "react";

import { cn } from "@/lib/utils";
import { useIsMobile } from "@/hooks/use-mobile";

import { Button } from "@/components/ui/button";

interface FeatureItem {
  image: string;
  title: string;
  description: string;
}

interface ControlsProps {
  handleNext: () => void;
  handlePrevious: () => void;
  isPreviousDisabled: boolean;
  isNextDisabled: boolean;
}

const Controls = ({
  handleNext,
  handlePrevious,
  isPreviousDisabled,
  isNextDisabled,
}: ControlsProps) => {
  return (
    <div className="hidden flex-col items-start gap-8 lg:flex">
      <Button
        variant="outline"
        size="icon"
        className="rounded-full !bg-background/50 hover:!bg-background/100 [&_svg:not([class*='size-'])]:size-6"
        onClick={handlePrevious}
        disabled={isPreviousDisabled}
      >
        <ChevronUp />
      </Button>
      <Button
        variant="outline"
        size="icon"
        className="rounded-full !bg-background/50 hover:!bg-background/100 [&_svg:not([class*='size-'])]:size-6"
        onClick={handleNext}
        disabled={isNextDisabled}
      >
        <ChevronDown />
      </Button>
    </div>
  );
};

interface FeatureCardProps {
  feature: FeatureItem;
  isActive: boolean;
  onClick: () => void;
}

const FeatureCard = ({ feature, isActive, onClick }: FeatureCardProps) => {
  const variants = {
    initial: {
      opacity: 0,
    },
    animate: {
      opacity: 1,
    },
    exit: {
      opacity: 0,
    },
  };

  return (
    <AnimatePresence mode="popLayout">
      <motion.div
        layout
        transition={{
          layout: {
            duration: 0.4,
            ease: "easeOut",
          },
        }}
        style={{
          borderRadius: "24px",
        }}
        className="flex cursor-pointer items-start gap-4 overflow-hidden bg-background md:w-fit md:max-w-sm"
        onClick={onClick}
      >
        {isActive ? (
          <motion.div
            layout
            variants={variants}
            initial="initial"
            animate="animate"
            exit="exit"
            key={`feature-description-active-${feature.title}`}
            transition={{
              duration: 0.4,
              delay: 0.3,
              ease: "easeOut",
            }}
            className="p-6 text-sm md:p-8 md:text-base"
          >
            <p>
              <span className="font-semibold">{feature.title}.</span>{" "}
              <span>{feature.description}</span>
            </p>
          </motion.div>
        ) : (
          <motion.div
            layout
            variants={variants}
            initial="initial"
            animate="animate"
            exit="exit"
            key={`feature-description-inactive-${feature.title}`}
            transition={{
              duration: 0.4,
              delay: 0.2,
              ease: "easeOut",
            }}
            className={cn(
              "flex h-fit shrink-0 items-center gap-4 text-sm md:py-3.5 md:pr-6 md:pl-3 md:text-base",
              !isActive && "h-0 w-0 md:h-auto md:w-auto",
            )}
            style={{
              height: "auto",
              lineHeight: "normal",
            }}
          >
            <PlusCircle strokeWidth={1.5} />
            <p className="shrink-0 font-semibold">{feature.title}</p>
          </motion.div>
        )}
      </motion.div>
    </AnimatePresence>
  );
};

interface FeaturesDesktopProps {
  features: FeatureItem[];
  handleNext: () => void;
  handlePrevious: () => void;
  activeIndex: number;
  handleFeatureClick: (index: number) => void;
  isPreviousDisabled: boolean;
  isNextDisabled: boolean;
}

const FeaturesDesktop = ({
  features,
  handleNext,
  handlePrevious,
  activeIndex,
  handleFeatureClick,
  isPreviousDisabled,
  isNextDisabled,
}: FeaturesDesktopProps) => {
  return (
    <div className="relative z-10 hidden items-center gap-8 md:flex">
      <Controls
        handleNext={handleNext}
        handlePrevious={handlePrevious}
        isPreviousDisabled={isPreviousDisabled}
        isNextDisabled={isNextDisabled}
      />
      <div className="flex flex-col gap-4">
        {features.map((feature, index) => {
          return (
            <FeatureCard
              key={`feature-card-${index}`}
              feature={feature}
              isActive={index === activeIndex}
              onClick={() => handleFeatureClick(index)}
            />
          );
        })}
      </div>
    </div>
  );
};

interface FeatureMobileProps {
  features: FeatureItem[];
  handleNext: () => void;
  handlePrevious: () => void;
  activeIndex: number;
  direction: 1 | -1;
  isPreviousDisabled: boolean;
  isNextDisabled: boolean;
}

const FeaturesMobile = ({
  features,
  handleNext,
  handlePrevious,
  activeIndex,
  direction,
  isPreviousDisabled,
  isNextDisabled,
}: FeatureMobileProps) => {
  const variants = {
    initial: (direction: 1 | -1) => ({
      opacity: 0,
      scale: 0.6,
      x: direction * 50 + "%",
    }),
    animate: {
      opacity: 1,
      scale: 1,
      x: 0,
    },
    exit: (direction: 1 | -1) => ({
      opacity: 0,
      scale: 0.6,
      x: direction * -50 + "%",
    }),
  };

  return (
    <div className="absolute bottom-6 left-0 z-10 flex w-full items-end justify-between gap-6 px-6 md:hidden">
      <Button
        variant="outline"
        size="icon"
        className="rounded-full !bg-background/100 [&_svg:not([class*='size-'])]:size-6"
        onClick={handlePrevious}
        disabled={isPreviousDisabled}
      >
        <ChevronLeft />
      </Button>
      <AnimatePresence mode="popLayout" custom={direction}>
        <motion.div
          key={`feature-mobile-${activeIndex}`}
          variants={variants}
          initial="initial"
          animate="animate"
          exit="exit"
          custom={direction}
          transition={{
            duration: 0.2,
            ease: "easeOut",
          }}
          className="h-full w-full object-cover"
        >
          <FeatureCard
            feature={features[activeIndex]}
            isActive={true}
            onClick={() => {}}
          />
        </motion.div>
      </AnimatePresence>

      <Button
        variant="outline"
        size="icon"
        className="rounded-full !bg-background/100 [&_svg:not([class*='size-'])]:size-6"
        onClick={handleNext}
        disabled={isNextDisabled}
      >
        <ChevronRight />
      </Button>
    </div>
  );
};

interface FeatureStepperProps {
  heading?: string;
  features?: FeatureItem[];
  className?: string;
}

const FeatureStepper = ({
  className,
  heading = "Discover Our Products",
  features = [
    {
      image:
        "https://deifkwefumgah.cloudfront.net/shadcnblocks/block/ecommerce/electronics/klim-musalimov-DAGss-dkVOs-unsplash-2.jpg",
      title: "Smart Watches",
      description:
        "Stay connected and track your health with advanced fitness monitoring. Customizable watch faces for every lifestyle.",
    },
    {
      image:
        "https://deifkwefumgah.cloudfront.net/shadcnblocks/block/ecommerce/electronics/klim-musalimov-IGO10LkxP_g-unsplash-2.jpg",
      title: "Fitness Tracking",
      description:
        "Monitor your daily activity, sleep patterns, and workout performance. Water-resistant design with week-long battery life.",
    },
    {
      image:
        "https://deifkwefumgah.cloudfront.net/shadcnblocks/block/ecommerce/electronics/dennis-cortes-cL7xovIO7sw-unsplash-3.jpg",
      title: "Gaming Consoles",
      description:
        "Next-generation gaming with stunning 4K graphics, lightning-fast load times, and an expansive library of exclusive titles.",
    },
    {
      image:
        "https://deifkwefumgah.cloudfront.net/shadcnblocks/block/ecommerce/electronics/Sleek-Laptop-Silhouette-1.png",
      title: "Premium Laptops",
      description:
        "Powerful performance in a sleek design. Latest processors, all-day battery life, and stunning displays for work and play.",
    },
    {
      image:
        "https://deifkwefumgah.cloudfront.net/shadcnblocks/block/ecommerce/other/jascent-leung-fCO3tBcnkhg-unsplash-1.jpg",
      title: "Phone Cases & Accessories",
      description:
        "Protect your device with premium cases, wireless charging stands, and accessories that combine style with functionality.",
    },
    {
      image: "https://deifkwefumgah.cloudfront.net/shadcnblocks/block/ecommerce/electronics/Hands-Typing-on-Laptop-1.png",
      title: "Work From Anywhere",
      description:
        "High-performance laptops built for professionals. Reliable computing power, comfortable keyboards, and enterprise-grade security.",
    },
  ],
}: FeatureStepperProps) => {
  const [activeIndex, setActiveIndex] = useState(0);
  const [direction, setDirection] = useState<1 | -1>(1);

  const isMobile = useIsMobile();

  const handleNext = () => {
    setDirection(1);
    if (activeIndex !== features.length - 1) {
      setActiveIndex((prevIndex) => prevIndex + 1);
    }
  };

  const handlePrevious = () => {
    setDirection(-1);
    if (activeIndex !== 0) {
      setActiveIndex((prevIndex) => prevIndex - 1);
    }
  };

  const handleFeatureClick = (index: number) => {
    setDirection(index > activeIndex ? 1 : -1);
    setActiveIndex(index);
  };

  const xOffset = !isMobile ? 50 : 15;
  const yOffset = !isMobile ? 15 : 5;
  const scale = !isMobile ? 0.6 : 0.8;

  const variants = {
    initial: (direction: 1 | -1) => ({
      opacity: 0,
      scale: scale,
      filter: "blur(20px)",
      x: direction * xOffset + "%",
      y: direction * yOffset + "%",
    }),
    animate: {
      opacity: 1,
      scale: 1,
      filter: "blur(0px)",
      x: 0,
      y: 0,
    },
    exit: (direction: 1 | -1) => ({
      opacity: 0,
      scale: scale,
      x: direction * -xOffset + "%",
      y: direction * -yOffset + "%",
      filter: "blur(20px)",
    }),
  };

  return (
    <section className={cn("mx-auto w-full max-w-5xl px-6 pb-20 md:px-8", className)}>
      <div className="flex w-full flex-col items-center justify-center space-y-20">
        <div className="flex flex-col items-center justify-center max-w-5xl">
          <h2 className="text-2xl font-semibold sm:text-5xl">{heading}</h2>
        </div>
        <div className="relative flex h-full min-h-[60vh] w-full overflow-hidden rounded-4xl bg-muted px-8 py-8 md:min-h-full md:py-20">
          <FeaturesDesktop
            features={features}
            handleNext={handleNext}
            handlePrevious={handlePrevious}
            activeIndex={activeIndex}
            handleFeatureClick={handleFeatureClick}
            isPreviousDisabled={activeIndex === 0}
            isNextDisabled={activeIndex === features.length - 1}
          />
          <FeaturesMobile
            features={features}
            handleNext={handleNext}
            handlePrevious={handlePrevious}
            activeIndex={activeIndex}
            direction={direction}
            isPreviousDisabled={activeIndex === 0}
            isNextDisabled={activeIndex === features.length - 1}
          />

          <div className="absolute top-0 right-0 z-0 flex h-full w-full items-center justify-center lg:w-2/3 lg:mask-[linear-gradient(to_right,_transparent,_black_30%,_black)]">
            <AnimatePresence mode="popLayout" custom={direction}>
              <motion.img
                key={`feature-image-${activeIndex}`}
                variants={variants}
                initial="initial"
                animate="animate"
                exit="exit"
                custom={direction}
                transition={{
                  duration: 0.4,
                  ease: "easeOut",
                }}
                src={features[activeIndex].image}
                alt={features[activeIndex].title}
                className="h-full w-full object-cover"
              />
            </AnimatePresence>
          </div>
        </div>
      </div>
    </section>
  );
};

export { FeatureStepper };
