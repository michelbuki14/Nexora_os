import { BoxGeometry, MeshStandardMaterial, Mesh, Group, useRef } from "three";
import { useState } from "react";
import { useReducedMotion } from "../utils/use-reduced-motion";

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

  // Simple scale animation - respect reduced motion
  const hoveredScale = isHovered && !reducedMotion ? 1.1 : 1.0;

  return (
    <group
      scale={hoveredScale}
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
      <mesh rotation={[0, 0, 0]}>
        <boxGeometry args={[2, 1, 0.5]} />
        <meshStandardMaterial color={color} />
      </mesh>

      {/* Title area */}
      <group position={[0, 0.8, 0]} rotation={[-0.5, 0, 0]}>
        <mesh>
          <boxGeometry args={[2.2, 0.3, 0.1]} />
          <meshStandardMaterial color={color} transparent opacity={0.9} depthWrite={false} />
        </mesh>
        <mesh>
          <boxGeometry args={[2.1, 0.25, 0.05]} />
          <meshStandardMaterial color="#FFFFFF" transparent opacity={0.8} depthWrite={false} />
        </mesh>
      </group>

      {/* Description area */}
      <group position={[0, -0.3, 0]} rotation={[-0.5, 0, 0]}>
        <mesh>
          <boxGeometry args={[2.2, 0.3, 0.1]} />
          <meshStandardMaterial color="#FFFFFF" transparent opacity={0.6} depthWrite={false} />
        </mesh>
        <mesh>
          <boxGeometry args={[2.1, 0.25, 0.05]} />
          <meshStandardMaterial color="#000000" transparent opacity={0.5} depthWrite={false} />
        </mesh>
      </group>
    </group>
  );
};