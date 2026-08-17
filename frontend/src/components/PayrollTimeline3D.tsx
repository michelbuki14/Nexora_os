import { Group, Mesh, CylinderGeometry, MeshStandardMaterial, ConeGeometry, Text } from "three";
import { useSpring, useVelocity } from "@react-three/drei";
import { useState, useMemo } from "react";

interface PayrollRun {
  id: string;
  period: string;
  status: "draft" | "confirmed" | "paid";
  amount: number;
}

interface PayrollTimeline3DProps {
  payrollRuns: PayrollRun[];
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
    <group>
      {/* Timeline track */}
      <Mesh position={[0, 0, (payrollRuns.length - 1) * 0.6]} rotation={[-Math.PI / 2, 0, 0]}>
        <CylinderGeometry args={[0.05, 0.05, (payrollRuns.length - 1) * 1.2, 8]} />
        <MeshStandardMaterial color="#9CA3AF" depthWrite />
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
              <ConeGeometry
                args={[radius, radius, radius, 12]}
                radialSegments={12}
              />
              <MeshStandardMaterial
                color={STATUS_COLORS[run.status]}
                opacity={0.9}
                depthWrite
                transparent
              />
            </Mesh>

            {/* Base platform */}
            <Mesh position={[0, -0.1, 0]}>
              <CylinderGeometry args={[radius + 0.2, radius + 0.2, 0.1, 16]} />
              <MeshStandardMaterial color="#E5E7EB" depthWrite />
            </Mesh>

            {/* Status indicator ring */}
            <Mesh position={[0, 0.05, 0]} rotation={[-Math.PI / 2, 0, 0]}>
              <CylinderGeometry args={[radius + 0.3, radius + 0.3, 0.05, 16]} />
              <MeshStandardMaterial
                color={STATUS_COLORS[run.status]}
                opacity={0.5}
                depthWrite
                transparent
              />
            </Mesh>

            {/* Period label */}
            <Mesh position={[2, 0, z]}>
              <Text text={run.period} fontSize={0.35} color="#374151" />
            </Mesh>

            {/* Status label */}
            <Mesh position={[-2.5, 0, z]}>
              <Text text={STATUS_LABELS[run.status]} fontSize={0.3} color={STATUS_COLORS[run.status]} />
            </Mesh>

            {/* Amount label (on hover) */}
            {hovered === i && (
              <Mesh position={[0, radius + 1, 0]}>
                <Text
                  text={`${(run.amount / 1000).toFixed(0)}K CDF`}
                  fontSize={0.4}
                  anchorX="center"
                  color="#1F2937"
                />
              </Mesh>
            )}
          </Group>
        );
      })}
    </group>
  );
};