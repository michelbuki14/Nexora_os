import { Canvas } from "@react-three/fiber";
import { OrbitControls } from "@react-three/drei";
import { useRef, useEffect } from "react";
import { useReducedMotion } from "@/utils/use-reduced-motion";
import { useIsMobile } from "@/utils/is-mobile";

/**
 * Debounced resize handler using requestAnimationFrame
 * Prevents layout thrashing from rapid resize events
 */
const useDebouncedResize = (callback: () => void, wait = 150) => {
  let raf: number;
  let timeout: NodeJS.Timeout;

  return () => {
    if (timeout) {
      clearTimeout(timeout);
      timeout = undefined;
    }
    if (!raf) {
      raf = requestAnimationFrame(() => {
        callback();
        raf = undefined;
      });
    } else {
      timeout = setTimeout(() => {
        callback();
        timeout = undefined;
        raf = requestAnimationFrame(() => {
          callback();
          raf = undefined;
        });
      }, wait);
    }
  };
};

export const DesignCanvas = ({
  children,
  cameraPosition = [0, 0, 5],
}: {
  children: React.ReactNode;
  cameraPosition?: [number, number, number];
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const reducedMotion = useReducedMotion();
  const isMobile = useIsMobile();

  useEffect(() => {
    // Handle resize with debounce to prevent layout thrashing
    const handleResize = useDebouncedResize(() => {
      if (canvasRef.current) {
        canvasRef.current.width = window.innerWidth;
        canvasRef.current.height = window.innerHeight;
        canvasRef.current.style.width = "100%";
        canvasRef.current.style.height = "100%";
      }
    });

    handleResize();
    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
    };
  }, []);

  // Adjust lighting and camera for mobile
  const cameraPos = isMobile ? [0, 0, 3] : cameraPosition;
  const ambientIntensity = reducedMotion ? 1.0 : 0.6;
  const directionalIntensity = reducedMotion ? 0.5 : 0.8;
  const damping = isMobile ? 0.5 : 0.3;

  return (
    <Canvas
      ref={canvasRef}
      shadows
      camera={{ position: cameraPos }}
      style={{ width: "100%", height: "100%", display: "block" }}
    >
      {/* Orbit controls for interaction */}
      <OrbitControls
        enableZoom={true}
        enablePan={true}
        enableRotate={!isMobile}
        autoRotate={false}
        damping={damping}
        // Keyboard shortcuts for accessibility
        keys={{
          LEFT: "ArrowLeft",
          RIGHT: "ArrowRight",
          UP: "ArrowUp",
          BOTTOM: "ArrowDown",
        }}
      />

      {/* Ambient light - increased for reduced motion */}
      <ambientLight intensity={ambientIntensity} />

      {/* Directional lights for 3D depth */}
      <directionalLight
        position={[5, 5, 5]}
        intensity={directionalIntensity}
        shadow={true}
      />
      <directionalLight
        position={[-5, -5, -5]}
        intensity={directionalIntensity}
        shadow={true}
      />

      {/* Hemi light for better illumination */}
      <hemisphereLight skyColor="#7F8C8D" groundColor="#F3F4F6" intensity={0.3} />

      {/* Fog for atmospheric depth */}
      <fog args={[0.01, 5, "#F3F4F6"]} />

      {/* Children rendered in 3D space */}
      {children}
    </Canvas>
  );
};