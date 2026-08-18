import { Group, Mesh, TorusGeometry, MeshStandardMaterial } from "three";
import { useSpring, useVelocity, Text } from "@react-three/drei";
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
    <Group rotation={[-0.4, 0, 0]} scale={scale}>
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
              <torusGeometry
                args={[1.8, 0.4, 8, 32, startAngle, arc]}
              />
              <meshStandardMaterial
                color={country.color}
                opacity={0.9}
                depthWrite
                transparent
              />
            </Mesh>

            {/* Inner ring for depth */}
            <Mesh>
              <torusGeometry
                args={[1.4, 0.2, 8, 32, startAngle, arc]}
              />
              <meshStandardMaterial
                color={country.color}
                opacity={0.5}
                depthWrite
                transparent
              />
            </Mesh>

            {/* Label */}
            <Text
              position={[x * 1.5, 0, z * 1.5]}
              fontSize={0.35}
              anchorX="center"
              color="#374151"
            >
              {country.country}: {country.count}
            </Text>

            {/* Percentage on hover */}
            {hovered === i && (
              <Text
                position={[x * 1.5, 1, z * 1.5]}
                fontSize={0.5}
                anchorX="center"
                color="#1F2937"
              >
                {((country.count / total) * 100).toFixed(1)}%
              </Text>
            )}
          </Group>
        );
      })}

      {/* Center text */}
      <Mesh position={[0, 0, 0]} rotation={[-Math.PI / 2, 0, 0]}>
        <Text
          fontSize={0.5}
          anchorX="center"
          anchorY="middle"
          color="#1F2937"
        >
          {total} tenants
        </Text>
      </Mesh>
    </Group>
  );
};