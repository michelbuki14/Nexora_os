import { Group, Mesh, BoxGeometry, MeshStandardMaterial } from "three";
import { useSpring, useVelocity, Text } from "@react-three/drei";
import { useState, useMemo } from "react";

interface WorkforceBarChart3DProps {
  departments: string[];
  employeeCounts: number[];
}

export const WorkforceBarChart3D = ({ departments, employeeCounts }: WorkforceBarChart3DProps) => {
  const [hovered, setHovered] = useState(-1);
  const maxCount = useMemo(() => Math.max(...employeeCounts), [employeeCounts]);

  const scale = useSpring(
    hovered >= 0 ? 1.15 : 1.0,
    useVelocity(0.15)
  );

  return (
    <Group>
      {departments.map((dept, i) => {
        const count = employeeCounts[i];
        const height = (count / maxCount) * 3;
        const x = -4 + i * 1.8;

        return (
          <Group
            key={dept}
            position={[x, 0, 0]}
            scale={scale}
            onPointerEnter={() => setHovered(i)}
            onPointerLeave={() => setHovered(-1)}
          >
            {/* Bar */}
            <Mesh position={[0, height / 2, 0]}>
              <boxGeometry args={[1.2, height, 0.6]} />
              <meshStandardMaterial
                color="#4F46E5"
                opacity={0.9}
                depthWrite
                transparent
              />
            </Mesh>

            {/* Base platform */}
            <Mesh position={[0, -0.1, 0]}>
              <boxGeometry args={[1.4, 0.1, 0.8]} />
              <meshStandardMaterial color="#E5E7EB" depthWrite />
            </Mesh>

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
          </Group>
        );
      })}
    </Group>
  );
};