import { Group, Mesh, BoxGeometry, MeshStandardMaterial } from "three";
import { Text } from "@react-three/drei";
import { useState, useMemo } from "react";

interface WorkforceBarChart3DProps {
  departments: string[];
  employeeCounts: number[];
}

export const WorkforceBarChart3D = ({ departments, employeeCounts }: WorkforceBarChart3DProps) => {
  const [hovered, setHovered] = useState(-1);
  const maxCount = useMemo(() => Math.max(...employeeCounts), [employeeCounts]);

  // Simple scale on hover - respect reduced motion via CSS or context
  const hoveredScale = hovered >= 0 ? 1.15 : 1.0;

  return (
    <group>
      {departments.map((dept, i) => {
        const count = employeeCounts[i];
        const height = (count / maxCount) * 3;
        const x = -4 + i * 1.8;

        return (
          <group
            key={dept}
            position={[x, 0, 0]}
            scale={hoveredScale}
            onPointerEnter={() => setHovered(i)}
            onPointerLeave={() => setHovered(-1)}
          >
            {/* Bar */}
            <mesh position={[0, height / 2, 0]}>
              <boxGeometry args={[1.2, height, 0.6]} />
              <meshStandardMaterial
                color="#4F46E5"
                opacity={0.9}
                depthWrite
                transparent
              />
            </mesh>

            {/* Base platform */}
            <mesh position={[0, -0.1, 0]}>
              <boxGeometry args={[1.4, 0.1, 0.8]} />
              <meshStandardMaterial color="#E5E7EB" depthWrite />
            </mesh>

            {/* Label */}
            <Text
              position={[x, -2.2, 0.5]}
              fontSize={0.4}
              maxWidth={1.5}
              anchorX="center"
              color="#374151"
            >
              {dept}
            </Text>

            {/* Value label */}
            <Text
              position={[x, height + 0.5, 0]}
              fontSize={0.5}
              anchorX="center"
              color="#1F2937"
            >
              {count.toString()}
            </Text>
          </group>
        );
      })}
    </group>
  );
};