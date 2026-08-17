# Frontend 3D Transformation - Phase 4: Polish & Accessibility

## Goal
Complete the 3D frontend transformation with performance optimization, accessibility, mobile responsiveness, and celebration effects.

## Phase 4 Overview
Following Phases 1 (foundation), 2 (interactive cards), and 3 (data visualization), Phase 4 adds the finishing touches that make the 3D experience production-ready.

## Accessibility Implementation

### 1. Reduce-Motion Media Query
```tsx
// src/utils/use-reduced-motion.ts
import { useEffect, useState } from "react";

export const useReducedMotion = () => {
  const [reduced, setReduced] = useState(false);

  useEffect(() => {
    const mql = window.matchMedia("(prefers-reduced-motion: reduce)");
    setReduced(mql.matches);

    const listener = (e: MediaQueryListEvent) => setReduced(e.matches);
    mql.addEventListener("change", listener);

    return () => mql.removeEventListener("change", listener);
  }, []);

  return reduced;
};
```

### 2. Apply in DesignCanvas
```tsx
// src/components/DesignCanvas.tsx
import { useReducedMotion } from "@/utils/use-reduced-motion";

const reducedMotion = useReducedMotion();

// Conditional animations
<ambientLight intensity={reducedMotion ? 1.0 : 0.5} />
<directionalLight intensity={reducedMotion ? 0.5 : 0.8} />

// Product card hover scale
const cardScale = reducedMotion ? 1.0 : useSpring(1.1, useVelocity(0.1));
```

### 3. Keyboard Navigation
```tsx
// OrbitControls with keyboard shortcuts
<OrbitControls
  autoRotate={false}
  enableZoom={true}
  enablePan={true}
  enableRotate={true}
  keys={{
    rotate: ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"],
    zoom: ["=","-"],
    pan: ["W","A","S","D"],
  }}
/>
```

### 4. ARIA Labels on Key Elements
```tsx
// Product cards
<ProductCard3D
  aria-label={`Nexora OS ${title} service card`}
  /* ... */
/>

/// Bar chart
<mesh aria-label={`Bar chart: ${department} has ${count} employees`} />

/// Donut chart
<mesh aria-label={`Tenant distribution: ${country} has ${count} tenants`} />
```

## Mobile Responsiveness

### 1. Detect Mobile Devices
```tsx
// src/utils/is-mobile.ts
export const isMobile = () => {
  return /Mobi|Android|iPhone|iPad/.test(navigator.userAgent) 
    || window.innerWidth < 768;
};
```

### 2. Responsive DesignCanvas
```tsx
// DesignCanvas.tsx - Conditional settings
const isMobileView = window.innerWidth < 768;

return (
  <Canvas
    camera={{ position: isMobileView ? [0, 0, 3] : [0, 0, 8] }}
    style={{ width: "100%", height: isMobileView ? "300px" : "100%" }}
  >
    {/* Fewer cards on mobile */}
    {isMobileView ? (
      <ProductCard3D
        title="Workforce"
        description="Employee service"
        color="#4F46E5"
        /* ... */
      />
    ) : (
      // Full 3 card layout
      <div style={{ display: "flex", gap: "1rem", justifyContent: "center" }}>
        <ProductCard3D ... />
        <ProductCard3D ... />
        <ProductCard3D ... />
      </div>
    )}
  </Canvas>
);
```

### 3. Touch-Friendly OrbitControls
```tsx
// Reduce sensitivity on touch devices
<OrbitControls
  damping={isMobileView ? 0.5 : 0.3}
  autoRotate={false}
  // Disable rotate on mobile if needed
  enableRotate={!isMobileView}
/>
```

### 4. Fallback Grid Layout
```tsx
// Dashboard3D.tsx - Mobile fallback
{isMobileView ? (
  <div
    style={{
      display: "grid",
      gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))",
      gap: "1rem",
      padding: "1rem",
    }}
  >
    <ProductCard3D title="Workforce" ... />
    <ProductCard3D title="Payroll" ... />
    <ProductCard3D title="Payments" ... />
  </div>
) : (
  // 3D layout
  <DesignCanvas ...>
    {/* ... 3D content ... */}
  </DesignCanvas>
)}
```

## Confetti Celebration (Phase 4 Feature)

### 1. Install canvas-confetti (already in deps)
```bash
# Already installed from Phase 1
# canvas-confetti @types/canvas-confetti
```

### 2. Confetti Component
```tsx
// src/components/ConfettiCelebration.tsx
import confetti from "canvas-confetti";

export const ConfettiCelebration = ({
  trigger,
  duration = 3000,
}: {
  trigger: boolean;
  duration?: number;
}) => {
  useEffect(() => {
    if (trigger) {
      const count = 200;
      const defaults = { origin: { y: 0.6 } };

      function shootParticle(particle, defaults) {
        particle.velocity = {
          x: (Math.random() - 0.5) * 10,
          y: (Math.random() - 0.5) * 10 + 3,
          z: 0,
        };
        particle.gravity = 0.3;
        particle.friction = 0.5;
        particle.timeOut = Math.random() * 1000 + 500;
      }

      const particles = [];
      for (let i = 0; i < count; i++) {
        const particle = {};
        shootParticle(particle, defaults);
        particles.push(particle);
      }

      // Fire multiple times for different colors
      setTimeout(() => confetti({ origin: defaults.origin, particles }), 0);
      setTimeout(() => confetti({ origin: { ...defaults.origin, x: 0.2 }, particles }, 100);
      setTimeout(() => confetti({ origin: { ...defaults.origin, x: -0.2 }, particles }, 200);
      setTimeout(() => confetti({ origin: { ...defaults.origin, y: 0.2 }, particles }, 300);

      // Stop after duration
      const stopTime = setTimeout(() => {
        // Confetti auto-cleans after animation
      }, duration);

      return () => clearTimeout(stopTime);
    }
  }, [trigger]);

  return null;
};
```

### 3. Use in Dashboard3D
```tsx
// After successful task completion
<ConfettiCelebration trigger={showConfetti} />

// Example: after employee contract signed
const [showConfetti, setShowConfetti] = useState(false);

const handleContractSigned = () => {
  // ... save contract
  setShowConfetti(true);
  setTimeout(() => setShowConfetti(false), 3000);
};
```

## Performance Optimization

### 1. InstancedMesh for Repeated Geometry
```tsx
// Instead of individual meshes for many similar bars
import { InstancedMesh, BufferAttribute } from "three";

const barGeometry = new BoxGeometry(0.8, 1, 0.5);
const barMaterial = new MeshStandardMaterial({ color: "#4F46E5" });

const instancedBars = new InstancedMesh(barGeometry, barMaterial, employeeCounts.length);
instancedBars.instanceMatrix.setUsage(THREE.DynamicDrawUsage);

// Set positions
employeeCounts.forEach((count, i) => {
  const matrix = new THREE.Matrix4();
  matrix.makeTranslation(-4 + i * 1.5, count / 10, 0);
  instancedBars.setMatrixAt(i, matrix);
});

// In render
<instancedBars attach="mesh" />
```

### 2. Geometry Disposal
```tsx
// In useEffect cleanup
useEffect(() => {
  return () => {
    barGeometry.dispose();
    barMaterial.dispose();
    instancedBars.geometry.dispose();
    instancedBars.material.dispose();
  };
}, []);
```

### 3. RequestAnimationFrame Throttling
```tsx
// Only update when needed, not every frame
let lastUpdate = 0;

const animationLoop = (time: number) => {
  if (time - lastUpdate > 16) { // ~60fps
    // Update state
    lastUpdate = time;
  }
  requestAnimationFrame(animationLoop);
};

useEffect(() => {
  requestAnimationFrame(animationLoop);
  return () => cancelAnimationFrame(animationLoop);
}, []);
```

### 4. LOD (Level of Detail)
```tsx
// Simple LOD example
const lod = new THREE.LOD();

// Close view - detailed
lod.addLevel(detailedMesh, 0);

// Medium view - simplified
lod.addLevel(simplifiedMesh, 5);

// Far view - bounding box
lod.addLevel(boundingBoxMesh, 10);

scene.add(lod);
```

## Testing Checklist

### Accessibility Tests
- [ ] `prefers-reduced-motion` disables all animations
- [ ] Keyboard focus reaches OrbitControls
- [ ] ARIA labels present on all data visualization elements
- [ ] Color contrast meets WCAG AA (use existing tokens.css colors)
- [ ] Text alternatives for charts and graphs

### Performance Tests
- [ ] FPS > 50 on desktop (Chrome DevTools Performance)
- [ ] FPS > 30 on mobile (safari/Chrome dev tools)
- [ ] Memory usage stable (no leaks in DevTools)
- [ ] Page load < 2s on 3G simulation
- [ ] No JS errors in console

### Functional Tests
- [ ] OrbitControls: drag to rotate, scroll to zoom
- [ ] Product cards: hover scale, click navigation
- [ ] Bar charts: correct heights based on data
- [ ] Timeline: cylinders positioned along Z-axis
- [ ] Donut: arc proportional to data values
- [ ] Confetti: triggers on event, auto-cleans

### Mobile Tests
- [ ] Layout switches to grid at < 768px
- [ ] OrbitControls disabled or reduced sensitivity
- [ ] Touch events work (pinch to zoom alternative)
- [ ] Text is readable without zoom
- [ ] Tap targets >= 44px high

## Files Created This Phase

- `src/utils/use-reduced-motion.ts` - React hook for reduce-motion preference
- `src/utils/is-mobile.ts` - Mobile device detection
- `src/components/ConfettiCelebration.tsx` - Celebration effect
- Updates to `src/components/DesignCanvas.tsx` - Accessibility props
- Updates to `src/pages/Dashboard3D.tsx` - Mobile responsiveness
- Updates to `src/components/ProductCard3D.tsx` - ARIA labels

## Timeline - Final Week

| Day | Deliverable |
|-----|-------------|
| Day 1 | Reduce-motion hook + apply to all components |
| Day 2 | Mobile detection + responsive layout |
| Day 3 | Confetti celebration component |
| Day 4 | Performance optimization (InstancedMesh, LOD) |
| Day 5 | Accessibility audit + testing |
| Day 6 | Bug fixes + polish |
| Day 7 | Documentation + deployment prep |

## How to Test the Complete 3D App

```bash
# Ensure all deps installed
bun install three @types/three @react-three/fiber @react-three/drei canvas-confetti @types/canvas-confetti

# Run dev server
bun run dev

# Visit http://localhost:5173/

# Test checklist:
# 1. Page loads with 3D canvas
# 2. Product cards rotate and scale on hover
# 3. Drag to rotate the scene (OrbitControls)
# 4. Scroll to zoom in/out
# 5. Click cards to see details
# 6. Resize browser < 768px - layout switches to grid
# 7. Open DevTools > Preferences > Reduced motion - animations stop
# 8. Open DevTools > Accessibility - ARIA labels present
# 9. Trigger confetti (e.g., click a milestone event)
# 10. Verify no console errors
# 11. Check mobile viewport - touch-friendly controls
```

## Final File Structure

```
frontend/src/
├── components/
│   ├── DesignCanvas.tsx          ← With accessibility props
│   ├── ProductCard3D.tsx         ← With ARIA labels
│   ├── WorkforceBarChart3D.tsx   ← Phase 3
│   ├── PayrollTimeline3D.tsx     ← Phase 3
│   ├── TenantDonut3D.tsx         ← Phase 3
│   └── ConfettiCelebration.tsx   ← Phase 4 (NEW)
├── utils/
│   ├── use-reduced-motion.ts     ← Phase 4 (NEW)
│   └── is-mobile.ts              ← Phase 4 (NEW)
└── pages/
    └── Dashboard3D.tsx           ← Mobile responsive + all components
```

## Deployment Checklist

Before deploying to production:

- [ ] All 6 3D dependencies installed and `bun install` / `npm install` passed
- [ ] `bun run dev` works on port 5173
- [ ] Accessibility audit passed (keyboard, screen reader, color contrast)
- [ ] Performance profiled: FPS > 50 desktop, > 30 mobile
- [ ] Mobile responsiveness verified at < 768px
- [ ] No console errors in any browser
- [ ] Confetti works and auto-cleans
- [ ] Reduce-motion preference respected
- [ ] Build: `bun run build` or `npm run build` produces clean `dist/`
- [ ] `dist/index.html` references correct asset hashes
- [ ] API proxy still targets `http://localhost:8080`
- [ ] CHANGELOG updated with 3D feature release

## Release Notes (for CHANGELOG.md)

When shipping, add entry:

```
## [0.2.0] - 2026-08-XX

### Added
- **3D frontend transformation** — Full Three.js + React Three Fiber integration
- **Interactive product cards** — 3D meshes with hover scaling and rotation
- **Data visualization** — Bar charts, timeline donuts for workforce/payroll/tenant metrics
- **Accessibility** — reduce-motion preference, keyboard navigation, ARIA labels
- **Mobile responsiveness** — Grid fallback at < 768px, touch-friendly controls
- **Celebration effects** — canvas-confetti on milestone completion

### Changed
- `vite.config.ts` — Dev server port moved from 3000 to 5173
- `src/main.tsx` — Wrapped with `<DesignCanvas>`
- `src/app/routes.tsx` — Dashboard3D as default route

### Fixed
- [List any bugs fixed during development]

### Deprecated
- [Nothing deprecated in this release]