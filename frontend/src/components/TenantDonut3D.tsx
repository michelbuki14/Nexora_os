import { Group, Mesh, TorusGeometry, MeshStandardMaterial, Text } from "three";
import { useSpring, useVelocity } from "@react-three/drei";
import { useState, useMemo } from "react";

interface CountryData {
  country: string;
  count: number;
  color: string;
}

interface TenantDonut3DProps {
  countryData: CountryData[];
}

export const TenantDonut3D = ({ countryData }: TenantDonut3DProps) => {
  const [hovered, setHovered] = useState(-1);
  const total = useMemo(() => countryData.reduce((sum, c) => sum + c.count, 0), [countryData]);

  const scale = useSpring(
    hovered >= 0 ? 1.1 : 1.0,
    useVelocity(0.15)
  );

  return (
    <group rotation={[-0.4, 0, 0]} scale={scale}>
      {countryData.map((country, i) => {
        const previousCountries = countryData.slice(0, i);
        const startAngle = previousCountries.reduce((sum, c) => sum + (c.count / total) * Math.PI * 2, 0);
        const arc = (country.count / total) * Math.PI * 2;
        const midAngle = startAngle + arc / 2;
        const radius = 1.8;

        const x = Math.cos(midAngle) * radius;
        const z = Math.sin(midAngle) * radius;

        return (
          <Group
            key={country.country}
            onPointerEnter={() => setHovered(i)}
            onPointerLeave={() => setHovered(-1)}
          >
            {/* Donut segment */}
            <Mesh>
              <TorusGeometry
                radius={1.8}
                tube={0.4}
                radialSegments={8}
                tubularSegments={32}
                arc={arc}
                args={[1.8, 0.4, 8, 32, 0, arc]}
              />
              <MeshStandardMaterial
                color={country.color}
                opacity={0.9}
                depthWrite
                transparent
              />
            </Mesh>

            {/* Inner ring for depth */}
            <Mesh>
              <TorusGeometry
                radius={1.4}
                tube={0.2}
                radialSegments={8}
                tubularSegments={32}
                arc={arc}
              />
              <MeshStandardMaterial
                color={country.color}
                opacity={0.5}
                depthWrite
                transparent
              />
            </Mesh>

            {/* Label */}
            <Mesh position={[x * 1.5, 0, z * 1.5]}>
              <Text
                text={`${country.country}: ${country.count}`}
                fontSize={0.35}
                anchorX="center"
                color="#374151"
              />
            </Mesh>

            {/* Percentage on hover */}
            {hovered === i && (
              <Mesh position={[x * 1.5, 1, z * 1.5]}>
                <Text
                  text={`${((country.count / total) * 100).toFixed(1)}%`}
                  fontSize={0.5}
                  anchorX="center"
                  color="#1F2937"
                />
              </Mesh>
            )}
          </Group>
        );
      })}

      {/* Center text */}
      <Mesh position={[0, 0, 0]} rotation={[-Math.PI / 2, 0, 0]}>
        <Text text={`${total} tenants`} fontSize={0.5} anchorX="center" anchorY="middle" color="#1F2937" />
      </Mesh>
    </group>
  );
};