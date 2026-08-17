import { DesignCanvas } from "@/components/DesignCanvas";
import { ProductCard3D } from "@/components/ProductCard3D";
import { WorkforceBarChart3D } from "@/components/WorkforceBarChart3D";
import { PayrollTimeline3D } from "@/components/PayrollTimeline3D";
import { TenantDonut3D } from "@/components/TenantDonut3D";
import { ConfettiCelebration } from "@/components/ConfettiCelebration";
import { useState, useCallback, useEffect } from "react";
import { useReducedMotion } from "@/utils/use-reduced-motion";
import { useIsMobile, useViewport } from "@/utils/is-mobile";
import { useHasAnyPermission } from "@/auth/usePermission";

export const Dashboard3D = () => {
  const [selected, setSelected] = useState<string | null>(null);
  const [showConfetti, setShowConfetti] = useState(false);
  const reducedMotion = useReducedMotion();
  const isMobile = useIsMobile();
  const { height } = useViewport();

  // Permission-gated rendering
  const hasPermission = useHasAnyPermission([
    "workforce.read",
    "payroll.read",
    "finance.read",
  ]);

  // Fallback to Desktop if no permission
  if (!hasPermission) {
    // Render a minimal non-3D version
    return null; // Will render the regular Dashboard instead
  }

  const handleCardClick = useCallback((title: string) => {
    setSelected(title);
    // Trigger confetti only if not reduced motion
    if (!reducedMotion) {
      setShowConfetti(true);
      setTimeout(() => setShowConfetti(false), 3000);
    }
  }, [reducedMotion]);

  // Fetch real data on mount
  useEffect(() => {
    // In production, these would be actual API calls
    // getCurrentEmployeeCount().then(setEmployeeCount);
    // getPayrollRuns().then(setPayrollRuns);
    // getTenantDistribution().then(setCountryData);
  }, []);

  // Real data placeholders (would come from API state)
  const employeeCount = 124; // Would come from API
  const payrollRuns = []; // Would come from API
  const countryData = []; // Would come from API

  // If no real data loaded yet, show skeleton
  if (payrollRuns.length === 0 && countryData.length === 0) {
    return (
      <DesignCanvas
        cameraPosition={[0, 0, 8]}
        style={{ width: "100%", height: "100%" }}
      >
        <div style={{ position: "absolute", top: 20, left: 20, color: "#111827" }}>
          <h1 style={{ margin: 0, fontSize: "2rem" }}>Nexora OS Dashboard</h1>
          <p style={{ margin: "0.5rem 0 0", fontSize: "1rem", color: "#6B7280" }}>
            Loading 3D dashboard...
          </p>
        </div>
        <div style={{ position: "absolute", bottom: 40, left: 40, right: 40 }}>
          <ProductCard3D
            title="Workforce Service"
            description="Data loading..."
            color="#4F46E5"
            disabled
          />
        </div>
      </DesignCanvas>
    );
  }

  // Mobile fallback: grid layout instead of 3D
  if (isMobile) {
    return (
      <DesignCanvas cameraPosition={[0, 0, 3]} style={{ width: "100%", height: "300px" }}>
        <ConfettiCelebration trigger={showConfetti} reducedMotion={reducedMotion} />

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
            description={`{payrollRuns.length} runs`}
            color="#10B981"
          />
          <ProductCard3D
            title="Payments"
            description={`{Math.round(countryData.reduce((s, c) => s + c.count, 0))} tenants`}
            color="#F59E0B"
          />
        </div>

        {selected && (
          <div
            style={{
              position: "absolute",
              top: 80,
              left: 10,
              right: 10,
              background: "rgba(255, 255, 255, 0.95)",
              padding: "0.75rem",
              borderRadius: "0.5rem",
              color: "#111827",
              fontSize: "0.8rem",
              textAlign: "center",
            }}
          >
            <strong>{selected} service selected</strong>
          </div>
        )}
      </DesignCanvas>
    );
  }

  // Desktop: full 3D layout with reduced motion support
  return (
    <DesignCanvas
      cameraPosition={[0, 0, 8]}
      style={{ width: "100%", height: "100%" }}
    >
      <ConfettiCelebration
        trigger={showConfetti}
        reducedMotion={reducedMotion}
        colors={reducedMotion ? [] : ["#4F46E5", "#10B981", "#F59E0B"]}
      />

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
          gap: isMobile ? "1rem" : "1.5rem",
        }}
      >
        <ProductCard3D
          title="Workforce Service"
          description={`${employeeCount} employees`}
          color="#4F46E5"
          reducedMotion={reducedMotion}
        />
        <ProductCard3D
          title="Payroll Service"
          description={`${payrollRuns.length} runs`}
          color="#10B981"
          reducedMotion={reducedMotion}
        />
        <ProductCard3D
          title="Payments Service"
          description={`${countryData.length} regions`}
          color="#F59E0B"
          reducedMotion={reducedMotion}
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
        {payrollRuns.length > 0 && (
          <PayrollTimeline3D payrollRuns={payrollRuns} reducedMotion={reducedMotion} />
        )}
        {countryData.length > 0 && (
          <TenantDonut3D countryData={countryData} reducedMotion={reducedMotion} />
        )}
        {!payrollRuns.length && !countryData.length && (
          <div style={{ color: "#6B7280", fontSize: "0.9rem", textAlign: "center" }}>
            Data visualizations will appear here once loaded
          </div>
        )}
      </div>

      {/* Selected card details */}
      {selected && (
        <div
          style={{
            position: "absolute",
            top: 100,
            left: "50%",
            transform: "translateX(-50%)",
            background: "rgba(255, 255, 255, 0.95)",
            padding: "1rem 1.5rem",
            borderRadius: "0.5rem",
            color: "#111827",
            fontSize: "0.9rem",
            backdropFilter: "blur(10px)",
            boxShadow: "0 4px 20px rgba(0,0,0,0.1)",
            textAlign: "center",
            zIndex: 100,
          }}
        >
          <strong>{selected} service selected</strong>
          <p style={{ margin: "0.5rem 0 0", fontSize: "0.8rem", color: "#6B7280" }}>
            {reducedMotion
              ? "View details in list view"
              : "Click to explore 3D data visualizations"}
          </p>
        </div>
      )}
    </DesignCanvas>
  );
};