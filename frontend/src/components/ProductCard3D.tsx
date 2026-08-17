import { Mesh, BoxGeometry, MeshStandardMaterial, Group, useRef, useClock } from "three";
import { useEnter, useHover } from "@react-three/fiber";
import { useSpring, useVelocity } from "@react-three/drei";
import { useState } from "react";
import { useReducedMotion } from "@/utils/use-reduced-motion";

export const ProductCard3D = ({
  title,
  description,
  color = "#4F46E5",
  onClick,
}: {
  title: string;
  description: string;
  color?: string;
  onClick?: () => void;
}) => {
  const [isHovered, setIsHovered] = useState(false);
  const [isPressed, setIsPressed] = useState(false);
  const reducedMotion = useReducedMotion();
  const clock = useClock();

  // Spring-animated scale - respect reduced motion
  const scale = useSpring(
    isHovered && !reducedMotion ? 1.1 : 1.0,
    useVelocity(0.1)
  );

  // Rotation animation using useClock - more efficient than useFrame
  // Clock.getDelta() returns time since last frame in seconds
  const rotation = useRef(0);
  useFrame((state) => {
    if (!reducedMotion) {
      rotation.current += state.clock.getDelta() * 0.5;
    }
  });

  return (
    <group
      scale={scale}
      rotation={[-0.5, 0, 0]}
      onPointerEnter={() => setIsHovered(true)}
      onPointerLeave={() => setIsHovered(false)}
      onClick={() => onClick && onClick()}
      role="button"
      tabIndex={0}
      aria-label={`Nexora OS ${title} service card`}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onClick && onClick();
        }
      }}
    >
      <mesh rotation={rotation}>
        <boxGeometry args={[2, 1, 0.5]} />
        <meshStandardMaterial color={color} />
      </mesh>

      {/* Title area */}
      <group position={[0, 0.8, 0]} rotation={[-0.5, 0, 0]}>
        <meshStandardMaterial color={color} transparent opacity={0.9} depthWrite={false}>
          <boxGeometry args={[2.2, 0.3, 0.1]} />
        </meshStandardMaterial>
        <meshStandardMaterial color="#FFFFFF" transparent opacity={0.8} depthWrite={false}>
          <boxGeometry args={[2.1, 0.25, 0.05]} />
        </meshStandardMaterial>
      </group>

      {/* Description area */}
      <group position={[0, -0.3, 0]} rotation={[-0.5, 0, 0]}>
        <meshStandardMaterial color="#FFFFFF" transparent opacity={0.6} depthWrite={false}>
          <boxGeometry args={[2.2, 0.3, 0.1]} />
        </meshStandardMaterial>
        <meshStandardMaterial color="#000000" transparent opacity={0.5} depthWrite={false}>
          <boxGeometry args={[2.1, 0.25, 0.05]} />
        </meshStandardMaterial>
      </group>
    </group>
  );
};