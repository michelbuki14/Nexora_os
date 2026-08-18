import { useEffect } from "react";
import confetti from "canvas-confetti";
import { useReducedMotion } from "../utils/use-reduced-motion";

/**
 * SSR-safe confetti celebration
 * - Respects prefers-reduced-motion
 * - Reduces particle count on mobile
 * - Only runs in browser environment
 */
interface ConfettiCelebrationProps {
  trigger: boolean;
  duration?: number;
  colors?: string[];
  reducedMotion?: boolean;
}

export const ConfettiCelebration = ({
  trigger,
  duration = 3000,
  colors = ["#4F46E5", "#10B981", "#F59E0B", "#EF4444", "#8B5CF6", "#EC4899"],
  reducedMotion: propReducedMotion,
}: ConfettiCelebrationProps) => {
  const hookReducedMotion = useReducedMotion();
  const isBrowser = typeof window !== "undefined";
  const reducedMotion = hookReducedMotion || propReducedMotion;

  // Skip entirely if not in browser or reduced motion preferred
  if (!isBrowser || !trigger) return null;

  useEffect(() => {
    if (!trigger) return;

    // Reduced motion: minimal confetti or skip entirely
    // On mobile: fewer particles for performance
    const isMobile = navigator.maxTouchPoints > 0 || window.innerWidth < 768;
    const particleCount = reducedMotion ? 0 : isMobile ? 50 : 200;

    if (reducedMotion || particleCount === 0) {
      return; // Skip confetti entirely
    }

    const origin = { y: 0.6 };

    function shootParticle(particle: any) {
      particle.velocity = {
        x: (Math.random() - 0.5) * 10,
        y: (Math.random() - 0.5) * 10 + 3,
        z: 0,
      };
      particle.gravity = 0.3;
      particle.friction = 0.5;
      particle.timeOut = Math.random() * 1000 + 500;
    }

    const particles: any[] = [];
    for (let i = 0; i < particleCount; i++) {
      const particle = {};
      shootParticle(particle);
      particles.push(particle);
    }

    // Fire bursts only on desktop (mobile gets single burst)
    const burstCount = isMobile ? 1 : 4;
    const bursts = [];

    for (let b = 0; b < burstCount; b++) {
      const originOffset =
        isMobile || reducedMotion
          ? { x: 0, y: 0 }
          : {
              x: (b / 4) * 0.6 - 0.15,
              y: (b % 2 === 0 ? -0.1 : 0.1),
            };
      bursts.push({ origin: { ...origin, ...originOffset }, colors, scalar: 1.2 });
    }

    const timeouts = bursts.map((burst, index) =>
      setTimeout(() => {
        confetti({ ...burst, particles });
      }, index * (isMobile ? 50 : 100))
    );

    // Clean up after duration
    const stopTimeout = setTimeout(() => {
      // confetti auto-cleans after animation completes
    }, duration);

    return () => {
      timeouts.forEach(clearTimeout);
      clearTimeout(stopTimeout);
    };
  }, [trigger, duration, colors, reducedMotion]);

  return null;
};