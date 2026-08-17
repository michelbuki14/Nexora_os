import { createBrowserRouter } from "react-router-dom";
import { PlannedModule } from "../modules/PlannedModule";
import { Forbidden } from "../pages/Forbidden";
import { NotFound } from "../pages/NotFound";
import { Dashboard } from "../pages/Dashboard";
import { Dashboard3D } from "../pages/Dashboard3D";
import { AppShell } from "../shell/AppShell";
import { WorkforceOverview } from "../workforce/WorkforceOverview";
import { EmployeeDirectory } from "../workforce/employees/EmployeeDirectory";
import { EmployeeCreate } from "../workforce/employees/EmployeeCreate";
import { EmployeeProfile } from "../workforce/employees/EmployeeProfile";
import { OrgListView } from "../workforce/org/OrgListView";

/** Planned modules render the honest "not enabled in this tenant" state. */
const PLANNED_MODULES = [
  "payroll",
  "finance",
  "payments",
  "analytics",
  "security",
  "ai",
  "infrastructure",
  "integrations",
  "developer",
  "organizations",
  "audit",
  "admin",
];

export const router = createBrowserRouter([
  {
    path: "/",
    element: <AppShell />,
    children: [
      { index: true, element: <Dashboard /> },
      { path: "3d", element: <Dashboard3D />,},
      { path: "workforce/employees", element: <EmployeeDirectory /> },
      { path: "workforce/employees/new", element: <EmployeeCreate /> },
      { path: "workforce/employees/:ulid", element: <EmployeeProfile /> },
      { path: "workforce/:resource", element: <OrgListView /> },
      ...PLANNED_MODULES.map((module) => ({
        path: module,
        element: <PlannedModule module={module} />,
      })),
      { path: "403", element: <Forbidden /> },
      { path: "404", element: <NotFound /> },
    ],
  },
  { path: "*", element: <NotFound /> },
]);
