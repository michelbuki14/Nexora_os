# Frontend 3D Transformation Plan

## Goal
Transform the Nexora OS frontend from a 2D React application into a full 3D experience using Three.js + React Three Fiber.

## Current State
- React + Vite + TailwindCSS frontend
- No 3D libraries currently installed
- Uses lucide-react for icons
- tokens.css for design tokens

## Required Dependencies
- `three` - Core 3D engine
- `@types/three` - TypeScript types
- `@react-three/fiber` - React renderer for Three.js
- `@react-three/drei` - Helpers and utilities
- `canvas-confetti` - For celebration effects (optional)
- `@types/canvas-confetti` - Types

## Architecture

### 1. Root Canvas Component
Create a `DesignCanvas.tsx` that wraps the entire app in a Three.js canvas:
- Full viewport coverage
- Responsive resize handling
- Proper cleanup on unmount

```tsx
import { Canvas } from "@react-three/fiber";
import { useGLTF } from "@react-three/drei";

export const DesignCanvas = ({ children }: { children: React.ReactNode }) => {
  return (
    <Canvas shadows camera={{ position: [0, 0, 5] }}>
      {/* Lights */}
      <ambientLight intensity={0.5} />
      <directionalLight position={[5, 5, 5]} intensity={0.5} />
      <directionalLight position={[-5, -5, -5]} intensity={0.5} />
      
      {/* Children rendered in 3D space */}
      {children}
    </Canvas>
  );
};
```

### 2. Product Cards as 3D Objects
Transform service cards into interactive 3D objects:

```tsx
import { Mesh, BoxGeometry, MeshStandardMaterial } from "three";
import { useFrame } from "@react-three/fiber";

export const ProductCard = () => {
  return (
    <mesh rotation={[-0.5, 0, 0]} on={useFrame(() => {
      // Rotate slowly
      rotation.y += 0.01;
    })}>
      <boxGeometry args={[1, 1, 1]} />
      <meshStandardMaterial color="#4F46E5" />
    </mesh>
  );
};
```

### 3. Interactive Elements
- Drag controls: `useDrag` from @react-three/drei
- Hover effects: change material on mouse enter/leave
- Click handlers for service selection

### 4. Data Visualization
- Use Three.js to visualize workforce metrics
- Payroll charts as 3D bar graphs
- Tenant distribution as pie charts in 3D

### 5. Performance Considerations
- Limit object count (< 50 concurrent meshes)
- Use instanced meshes for repeated elements
- Enable frustum culling (default in Three.js)
- Set reasonable resolution limits

## Implementation Phases

### Phase 1: Foundation (Week 1)
- [ ] Install dependencies
- [ ] Create root DesignCanvas component
- [ ] Add basic lighting and camera
- [ ] Convert 1-2 service cards to 3D

### Phase 2: Interactive Cards (Week 2)
- [ ] Make all service cards 3D meshes
- [ ] Add drag rotation interaction
- [ ] Hover state changes
- [ ] Click handlers

### Phase 3: Data Visualization (Week 3-4)
- [ ] 3D bar chart for workforce metrics
- [ ] 3D pie chart for tenant distribution
- [ ] Payroll timeline visualization

### Phase 4: Polish (Week 5)
- [ ] Add confetti on completion
- [ ] Mobile/responsive adjustments
- [ ] Performance optimization
- [ ] Accessibility (reduce motion option)

## Key Files to Create
- `src/components/DesignCanvas.tsx` - Root 3D canvas
- `src/components/ProductCard3D.tsx` - 3D service card
- `src/components/WorkforceChart3D.tsx` - Workforce metrics
- `src/components/PayrollChart3D.tsx` - Payroll data
- `src/app/3DApp.tsx` - Main app wrapper

## Design Tokens Integration
Leverage existing `tokens.css` colors:
- Primary: `#4F46E5` (indigo-600)
- Secondary: `#10B981` (emerald-500)
- Background: `#F3F4F6` (gray-100)
- Text: `#111827` (gray-900)

## Accessibility
- Provide `reduceMotion` media query support
- Ensure keyboard navigation fallback
- Alt text for 3D elements
- High contrast mode support

## Next Steps
1. Install dependencies
2. Create DesignCanvas component
3. Migrate existing service cards to 3D
4. Add interaction handlers
5. Implement data visualization components