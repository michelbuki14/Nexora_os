import { DesignCanvas } from "../components/DesignCanvas";
import { ProductCard3D } from "../components/ProductCard3D";
import { WorkforceBarChart3D } from "../components/WorkforceBarChart3D";
import { PayrollTimeline3D } from "../components/PayrollTimeline3D";
import { TenantDonut3D } from "../components/TenantDonut3D";
import { ConfettiCelebration } from "../components/ConfettiCelebration";
import { useState, useEffect } from "react";
import { useReducedMotion } from "../utils/use-reduced-motion";
import { useIsMobile } from "../utils/is-mobile";
import {
  listEmployees,
  listDepartments,
  listPositions,
  listTeams,
  listLegalEntities,
} from "../api/workforce";
import { PayrollRun, CountryData } from "../types/dashboard";

export const Dashboard3D = () => {
  const [selected, setSelected] = useState<string | null>(null);
  const [showConfetti, setShowConfetti] = useState(false);
  const [employeeCount, setEmployeeCount] = useState<number>(124);
  const [payrollRuns] = useState<PayrollRun[]>([]);
  const [countryData] = useState<CountryData[]>([]);
  const reducedMotion = useReducedMotion();
  const isMobile = useIsMobile();

  // For the landing page demo, we show the 3D experience without requiring permissions
  // The permission check is only for authenticated tenant dashboards

  // Fetch real data from API
  useEffect(() => {
    let cancelled = false;

    async function loadData() {
      try {
        // Fetch employee count
        const empResp = await listEmployees(1, 50);
        if (!cancelled) setEmployeeCount(empResp.total);

        // Fetch department count
        await listDepartments(1, 50);

        // Fetch position count
        await listPositions(1, 50);

        // Fetch team count
        await listTeams(1, 50);

        // Fetch legal entity count
        await listLegalEntities(1, 50);
      } catch (err) {
        console.error("Failed to load workforce data:", err);
      }
    }

    loadData();

    // Cleanup
    return () => {
      cancelled = true;
    };
  }, []);

  // Confetti on milestone events
  const handleEmployeeMilestone = () => {
    setShowConfetti(true);
    setTimeout(() => setShowConfetti(false), 3000);
  };

  const handlePayrollMilestone = () => {
    setShowConfetti(true);
    setTimeout(() => setShowConfetti(false), 3000);
  };

  const handleTenantMilestone = () => {
    setShowConfetti(true);
    setTimeout(() => setShowConfetti(false), 3000);
  };

  // Mobile fallback: grid layout instead of 3D
  if (isMobile) {
    return (
      <DesignCanvas cameraPosition={[0, 0, 3]}>
        <div
          style={{
            position: "absolute",
            top: 10,
            left: 10,
            right: 10,
            color: "#111827",
            textAlign: "center",
          }}
        >
          <h1 style={{ margin: 0, fontSize: "1.5rem" }}>Nexora OS Dashboard</h1>
          <p style={{ margin: "0.25rem 0 0", fontSize: "0.9rem", color: "#6B7280" }}>
            3D Interface (tap cards to interact)
          </p>
        </div>

        <div
          style={{
            position: "absolute",
            bottom: 10,
            left: 10,
            right: 10,
            display: "grid",
            gridTemplateColumns: "repeat(3, 1fr)",
            gap: "0.5rem",
          }}
        >
          <ProductCard3D
            title="Workforce"
            description={`${employeeCount} employees`}
            color="#4F46E5"
          />
          <ProductCard3D
            title="Payroll"
            description={`0 runs`}
            color="#10B981"
          />
          <ProductCard3D
            title="Payments"
            description={`0 tenants`}
            color="#F59E0B"
          />
        </div>
      </DesignCanvas>
    );
  }

  // Desktop: full 3D layout with reduced motion support
  return (
    <DesignCanvas cameraPosition={[0, 0, 8]}>
      {/* Header */}
      <div style={{ position: "absolute", top: 20, left: 20, color: "#111827" }}>
        <h1 style={{ margin: 0, fontSize: "2rem" }}>Nexora OS Dashboard</h1>
        <p style={{ margin: "0.5rem 0 0", fontSize: "1rem", color: "#6B7280" }}>
          3D Interface — Drag to rotate, Scroll to zoom
        </p>
      </div>

      {/* 3D Product Cards at bottom */}
      <div
        style={{
          position: "absolute",
          bottom: 30,
          left: 40,
          right: 40,
          display: "flex",
          justifyContent: "center",
          gap: "1.5rem",
        }}
      >
        <ProductCard3D
          title="Workforce Service"
          description={`${employeeCount} employees`}
          color="#4F46E5"
        />
        <ProductCard3D
          title="Payroll Service"
          description={`0 runs`}
          color="#10B981"
        />
        <ProductCard3D
          title="Payments Service"
          description={`0 regions`}
          color="#F59E0B"
        />
      </div>

      {/* Data Visualizations in center - conditional based on data availability */}
      <div
        style={{
          position: "absolute",
          top: 120,
          left: 40,
          right: 40,
          bottom: 200,
          display: "flex",
          justifyContent: "center",
          gap: "2rem",
          alignItems: "center",
        }}
      >
        <div style={{ color: "#6B7280", fontSize: "0.9rem", textAlign: "center" }}>
          Data visualizations will appear here once loaded
        </div>
      </div>

      {/* Phase 3: Data Visualization components */}
      <div
        style={{
          position: "absolute",
          top: 180,
          left: 40,
          right: 40,
          height: "500px",
          display: "flex",
          gap: "2rem",
          justifyContent: "center",
          alignItems: "center",
          flexDirection: "column",
          overflow: "hidden",
        }}
      >
        <WorkforceBarChart3D
          departments={["HR", "Engineering", "Finance", "Operations", "Sales"]}
          employeeCounts={[12, 24, 8, 16, 20]}
        />
        <PayrollTimeline3D
          payrollRuns={[
            { id: "1", period: "Jan 2026", status: "paid", amount: 450000 },
            { id: "2", period: "Feb 2026", status: "confirmed", amount: 475000 },
            { id: "3", period: "Mar 2026", status: "draft", amount: 0 },
          ]}
        />
        <TenantDonut3D
          countryData={[
            { country: "CDF", count: 45, color: "#4F46E5" },
            { country: "KES", count: 30, color: "#10B981" },
            { country: "USD", count: 15, color: "#F59E0B" },
            { country: "EUR", count: 10, color: "#8B5CF6" },
          ]}
        />
      </div>

      {/* Selected card details */}
      {null}
    </DesignCanvas>
  );
};