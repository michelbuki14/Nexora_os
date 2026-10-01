import { useState, useEffect } from "react";

/**
 * Lightweight skeleton that renders while the app initializes.
 * Shows a loading indicator and app title — replaces the empty
 * "body text length: 0" blank page we saw during QA.
 */
export const LoadingScreen = () => {
  const [step, setStep] = useState(0);

  useEffect(() => {
    const t1 = setTimeout(() => setStep(1), 400);
    const t2 = setTimeout(() => setStep(2), 900);
    const t3 = setTimeout(() => setStep(3), 1400);
    return () => { clearTimeout(t1); clearTimeout(t2); clearTimeout(t3); };
  }, []);

  const messages = [
    "Connecting to services…",
    "Loading dashboard data…",
    "Preparing 3D interface…",
    "Ready",
  ];

  return (
    <div className="flex h-screen items-center justify-center bg-slate-50 px-4">
      <div className="text-center max-w-sm">
        <div className="text-3xl font-bold text-slate-800 mb-2">Nexora OS</div>
        <div className="text-sm text-secondary mb-4">{messages[step]}</div>
        <div className="flex justify-center gap-1">
          {[0, 1, 2].map((i) => (
            <div
              key={i}
              className="w-2 h-2 rounded-full bg-slate-400 animate-pulse"
              style={{ animationDelay: `${i * 200}ms` }}
            />
          ))}
        </div>
      </div>
    </div>
  );
};
