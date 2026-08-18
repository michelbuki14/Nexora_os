import { Group, Mesh, CylinderGeometry, MeshStandardMaterial, ConeGeometry } from "three";
import { useSpring, useVelocity, Text } from "@react-three/drei";
import { useState, useMemo } from "react";
import { PayrollRun } from "../types/dashboard";

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

export const PayrollTimeline3D = ({ payrollRuns }: PayrollTimeline3DProps) => {
  const [hovered, setHovered] = useState(-1);
  const maxAmount = useMemo(
    () => Math.max(...payrollRuns.map((r) => r.amount), 1),
    [payrollRuns]
  );

  const scale = useSpring(
    hovered >= 0 ? 1.2 : 1.0,
    useVelocity(0.15)
  );

  return (
    <Group>
      {/* Timeline track */}
      <Mesh
        position={[0, 0, (payrollRuns.length - 1) * 0.6]}
        rotation={[-Math.PI / 2, 0, 0]}
      >
        <cylinderGeometry
          args={[0.05, 0.05, (payrollRuns.length - 1) * 1.2, 8]}
        />
        <meshStandardMaterial color="#9CA3AF" depthWrite />
      </Mesh>

      {payrollRuns.map((run, i) => {
        const z = i * 1.2;
        const radius = Math.max(0.3, (run.amount / maxAmount) * 1.5);

        return (
          <Group
            key={run.id}
            position={[0, 0, z]}
            scale={scale}
            onPointerEnter={() => setHovered(i)}
            onPointerLeave={() => setHovered(-1)}
          >
            {/* Cone/Column representing amount */}
            <Mesh position={[0, radius / 2, 0]} rotation={[-Math.PI / 2, 0, 0]}>
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
            </Mesh>

            {/* Base platform */}
            <Mesh position={[0, -0.1, 0]}>
              <cylinderGeometry args={[radius + 0.2, radius + 0.2, 0.1, 16]} />
              <meshStandardMaterial color="#E5E7EB" depthWrite />
            </Mesh>

            {/* Status indicator ring */}
            <Mesh position={[0, 0.05, 0]} rotation={[-Math.PI / 2, 0, 0]}>
              <cylinderGeometry args={[radius + 0.3, radius + 0.3, 0.05, 16]} />
              <meshStandardMaterial
                color={STATUS_COLORS[run.status]}
                opacity={0.5}
                depthWrite
                transparent
              />
            </Mesh>

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
          </Group>
        );
      })}
    </Group>
  );
};