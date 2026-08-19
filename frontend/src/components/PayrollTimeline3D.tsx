import { Group, Mesh, CylinderGeometry, MeshStandardMaterial, ConeGeometry } from "three";
import { Text } from "@react-three/drei";
import { useState, useMemo } from "react";

interface PayrollTimeline3DProps {
  payrollRuns: PayrollRun[];
  reducedMotion?: boolean;
}

const STATUS_COLORS = {
  paid: "#10B981",
  confirmed: "#F59E0B",
  draft: "#EF4444",
};

const STATUS_LABELS = {
  paid: "PAID",
  confirmed: "CONFIRMED",
  draft: "DRAFT",
};

export const PayrollTimeline3D = ({ payrollRuns, reducedMotion = false }: PayrollTimeline3DProps) => {
  const [hovered, setHovered] = useState(-1);
  const maxAmount = useMemo(
    () => Math.max(...payrollRuns.map((r) => r.amount), 1),
    [payrollRuns]
  );

  // Simple scale on hover - respect reduced motion
  const hoveredScale = hovered >= 0 && !reducedMotion ? 1.2 : 1.0;

  return (
    <group scale={hoveredScale}>
      {/* Timeline track */}
      <mesh
        position={[0, 0, (payrollRuns.length - 1) * 0.6]}
        rotation={[-Math.PI / 2, 0, 0]}
      >
        <cylinderGeometry
          args={[0.05, 0.05, (payrollRuns.length - 1) * 1.2, 8]}
        />
        <meshStandardMaterial color="#9CA3AF" depthWrite />
      </mesh>

      {payrollRuns.map((run, i) => {
        const z = i * 1.2;
        const radius = Math.max(0.3, (run.amount / maxAmount) * 1.5);

        return (
          <group
            key={run.id}
            position={[0, 0, z]}
            onPointerEnter={() => setHovered(i)}
            onPointerLeave={() => setHovered(-1)}
          >
            {/* Cone/Column representing amount */}
            <mesh position={[0, radius / 2, 0]} rotation={[-Math.PI / 2, 0, 0]}>
              <coneGeometry
                args={[radius, radius, radius, 12]}
                radialSegments={12}
              />
              <meshStandardMaterial
                color={STATUS_COLORS[run.status]}
                opacity={0.9}
                depthWrite
                transparent
              />
            </mesh>

            {/* Base platform */}
            <mesh position={[0, -0.1, 0]}>
              <cylinderGeometry args={[radius + 0.2, radius + 0.2, 0.1, 16]} />
              <meshStandardMaterial color="#E5E7EB" depthWrite />
            </mesh>

            {/* Status indicator ring */}
            <mesh position={[0, 0.05, 0]} rotation={[-Math.PI / 2, 0, 0]}>
              <cylinderGeometry args={[radius + 0.3, radius + 0.3, 0.05, 16]} />
              <meshStandardMaterial
                color={STATUS_COLORS[run.status]}
                opacity={0.5}
                depthWrite
                transparent
              />
            </mesh>

            {/* Period label */}
            <Text position={[2, 0, z]} fontSize={0.35} color="#374151">
              {run.period}
            </Text>

            {/* Status label */}
            <Text
              position={[-2.5, 0, z]}
              fontSize={0.3}
              color={STATUS_COLORS[run.status]}
            >
              {STATUS_LABELS[run.status]}
            </Text>

            {/* Amount label (on hover) */}
            {hovered === i && (
              <Text
                position={[0, radius + 1, 0]}
                fontSize={0.4}
                anchorX="center"
                color="#1F2937"
              >
                {(run.amount / 1000).toFixed(0)}K CDF
              </Text>
            )}
          </group>
        );
      })}
    </group>
  );
};