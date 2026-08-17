# Frontend 3D Transformation - Phase 3: Data Visualization

## Goal
Add 3D data visualization components to display workforce metrics, payroll data, and tenant distribution in Three.js.

## Phase 3 Overview
Following the foundation (Phase 1) and interactive cards (Phase 2), Phase 3 adds meaningful data representations in 3D space.

## Components to Create

### 1. Workforce Bar Chart (`src/components/WorkforceBarChart3D.tsx`)
3D bar chart showing employee count per department:

```tsx
import { Canvas, useFrame } from "@react-three/fiber";
import { Mesh, BoxGeometry, MeshStandardMaterial, Group } from "three";
import { useSpring, useVelocity } from "@react-three/drei";
import { useState } from "react";

export const WorkforceBarChart3D = ({
  departments,
  employeeCounts,
}: {
  departments: string[];
  employeeCounts: number[];
}) => {
  const [hovered, setHovered] = useState(-1);
  const scale = useSpring(
    hovered >= 0 ? 1.2 : 1.0,
    useVelocity(0.15)
  );

  return (
    <group>
      {departments.map((dept, i) => (
        <Group
          key={dept}
          position={[-4 + i * 1.5, 0, 0]}
          scale={scale}
          onHover={() => setHovered(i)}
          onHoverEnd={() => setHovered(-1)}
        >
          <mesh>
            <boxGeometry
              args={[
                0.8,
                (employeeCounts[i] / Math.max(...employeeCounts)) * 3,
                0.5,
              ]}
            />
            <meshStandardMaterial
              color="#10B981"
              opacity={0.8}
              depthWrite
            />
          </mesh>

          <meshStandardMaterial
            transparent
            opacity={0.5}
            depthWrite={false}
          >
            <boxGeometry args={[0.8, 0.1, 0.1]} />
          </meshStandardMaterial>

          <text position={[-4 + i * 1.5, -3.5, 0]}>
            {dept}
          </text>
        </Group>
      ))}
    </group>
  );
};
```

### 2. Payroll Timeline (`src/components/PayrollTimeline3D.tsx`)
Visualize payroll runs along Z-axis:

```tsx
import { Mesh, CylinderGeometry, MeshStandardMaterial } from "three";

export const PayrollTimeline3D = ({
  payrollRuns,
}: {
  payrollRuns: Array<{
    id: string;
    period: string;
    status: "draft" | "confirmed" | "paid";
    amount: number;
  }>;
}) => {
  return (
    <group>
      {payrollRuns.map((run, i) => (
        <mesh key={run.id} position={[0, 0, i * 1.2]} rotation={[-Math.PI / 2, 0, 0]}>
          <cylinderGeometry
            topRadius: 0
            bottomRadius={run.amount / 10000}
            radiusSegments: 12
            height: 0.5
          />
          <meshStandardMaterial
            color={run.status === "paid" ? "#10B981" : run.status === "confirmed" ? "#F59E0B" : "#EF4444"}
          />
        </mesh>
      ))}
    </group>
  );
};
```

### 3. Tenant Distribution Donut (`src/components/TenantDonut3D.tsx`)
3D donut chart showing tenant distribution by country:

```tsx
import { Mesh, TorusGeometry, MeshStandardMaterial } from "three";

export const TenantDonut3D = ({
  countryData,
}: {
  countryData: Array<{
    country: string;
    count: number;
    color: string;
  }>;
}) => {
  return (
    <group rotation={[-0.3, 0, 0]}>
      {countryData.map((country, i) => (
        <mesh key={country.country} position={[
          0,
          Math.sin((i / countryData.length) * Math.PI) * 2,
          Math.cos((i / countryData.length) * Math.PI) * 2,
        ]}>
          <torusGeometry
            radius: 1.5
            tube: 0.3
            radialSegments: 16
            tubularSegments: 32
            arc: (country.count / 100) * Math.PI * 2
          />
          <meshStandardMaterial color={country.color} />
        </mesh>
      ))}
    </group>
  );
};
```

## Integration into Dashboard3D

Add import and render components:

```tsx
import { WorkforceBarChart3D } from "@/components/WorkforceBarChart3D";
import { PayrollTimeline3D } from "@/components/PayrollTimeline3D";
import { TenantDonut3D } from "@/components/TenantDonut3D";

export const Dashboard3D = () => {
  // ... existing code

  return (
    <DesignCanvas cameraPosition={[0, 0, 8]} style={{ width: "100%", height: "100%" }}>
      {/* ... existing header and cards ... */}

      {/* Phase 3: Data Visualization */}
      <div
        style={{
          position: "absolute",
          top: 200,
          left: 40,
          right: 40,
          height: "400px",
          display: "flex",
          gap: "1.5rem",
          justifyContent: "center",
        }}
      >
        <WorkforceBarChart3D
          departments={["HR", "Engineering", "Finance", "Operations", "Sales"]}
          employeeCounts={[12, 24, 8, 16, 20]}
        />
        <PayrollTimeline3D
          payrollRuns={[
            { id: "1", period: "Jan 2026", status: "paid", amount: 450000 },
            { id: "2", period: "Feb 2026", status: "confirmed", amount: 475000 },
            { id: "3", period: "Mar 2026", status: "draft", amount: 0 },
          ]}
        />
        <TenantDonut3D
          countryData={[
            { country: "CDF", count: 45, color: "#4F46E5" },
            { country: "KES", count: 30, color: "#10B981" },
            { country: "USD", count: 15, color: "#F59E0B" },
            { country: "EUR", count: 10, color: "#8B5CF6" },
          ]}
        />
      </div>

      {/* ... rest */}
    </DesignCanvas>
  );
};
```

## Visual Styling

### Color Mapping (from tokens.css)
| Purpose | Color | CSS Variable |
|---------|-------|--------------|
| Primary bars | `#4F46E5` | indigo-600 |
| Secondary bars | `#10B981` | emerald-500 |
| Accent bars | `#F59E0B` | amber-500 |
| Text | `#111827` | gray-900 |
| Background | `#F3F4F6` | gray-100 |

### Lighting Adjustments for Data Viz
- Increase ambient light to `0.8` for better visibility of flat shapes
- Add a hemi light for soft shadows: `<hemisphereLight skyColor="#7F8C8D" groundColor="#F3F4F6" intensity={0.4} />`
- Keep directional lights from DesignCanvas

## Performance Considerations

### Object Limits
- Bar chart: max 20 bars (one per department)
- Timeline: max 50 payroll runs (virtualize older ones)
- Donut: max 12 countries (beyond becomes cluttered)

### Optimization Techniques
1. **InstancedMesh** for repetitive geometry (e.g., many similar bars)
2. **Frustum culling** (enabled by default in Three.js)
3. **LOD (Level of Detail)** - reduce detail when zoomed out
4. **RequestAnimationFrame throttling** - only update when needed

### Memory Management
- Dispose geometries/materials on unmount
- Use `useFrame` sparingly - only for animations, not static positioning
- Clean up event listeners in `useEffect` cleanup

## Accessibility

### Reduce Motion
```tsx
const prefersReducedMotion = window.matchMedia(
  "(prefers-reduced-motion: reduce)"
);

if (prefersReducedMotion.matches) {
  // Disable animations, keep static positions
  scale.stop();
}
```

### Keyboard Navigation
- OrbitControls supports:
  - `Alt + drag` = rotate
  - `Shift + drag` = pan
  - `Scroll` = zoom
  - `Arrow keys` = when canvas is focused

### ARIA labels
```tsx
<mesh aria-label={`Bar chart showing employee counts by department`} />
```

## Next Steps - Phase 4: Polish

After Phase 3 data visualization, Phase 4 will focus on:

1. **Mobile responsiveness** - reduce 3D complexity on small screens
2. **Confetti celebration** - canvas-confetti on task completion
3. **Reduce-motion preference** - respect user settings
4. **Confetti on completion** - celebrate milestones
5. **Accessibility audit** - keyboard navigation, screen reader support
6. **Performance profiling** - FPS target < 60 on target devices

## Timeline

| Week | Deliverable |
|------|-------------|
| Week 3 | WorkforceBarChart3D + PayrollTimeline3D |
| Week 3.5 | TenantDonut3D + Integration |
| Week 4 | Accessibility + Performance |
| Week 4.5 | Mobile responsiveness |
| Week 5 | Polish + Documentation |

## Files Created This Phase

- `src/components/WorkforceBarChart3D.tsx` - Department bar chart
- `src/components/PayrollTimeline3D.tsx` - Payroll run timeline
- `src/components/TenantDonut3D.tsx` - Tenant distribution donut
- Updates to `src/pages/Dashboard3D.tsx` - Integration

## How to Test

```bash
# After installing deps
bun run dev

# Visit
http://localhost:5173/

# Verify:
# - 3D bars rotate and respond to hover
# - Payroll timeline shows runs along Z-axis
# - Donut chart displays tenant countries
# - reduce-motion media query works
# - OrbitControls function (drag to rotate)
```