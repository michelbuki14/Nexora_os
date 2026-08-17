import { Group, Mesh, BoxGeometry, MeshStandardMaterial, Text } from "three";
import { useSpring, useVelocity } from "@react-three/drei";
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
    <group>
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
              <BoxGeometry args={[1.2, height, 0.6]} />
              <MeshStandardMaterial
                color="#4F46E5"
                opacity={0.9}
                depthWrite
                transparent
              />
            </Mesh>

            {/* Base platform */}
            <Mesh position={[0, -0.1, 0]}>
              <BoxGeometry args={[1.4, 0.1, 0.8]} />
              <MeshStandardMaterial color="#E5E7EB" depthWrite />
            </Mesh>

            {/* Label */}
            <Mesh position={[x, -2.2, 0.5]}>
              <Text text={dept} fontSize={0.4} maxWidth={1.5} anchorX="center" color="#374151" />
            </Mesh>

            {/* Value label */}
            <Mesh position={[x, height + 0.5, 0]}>
              <Text text={count.toString()} fontSize={0.5} anchorX="center" color="#1F2937" />
            </Mesh>
          </Group>
        );
      })}
    </group>
  );
};