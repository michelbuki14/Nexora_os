import { createBrowserRouter, Navigate } from "react-router-dom";
import { DemoModule } from "../modules/DemoModule";
import { Forbidden } from "../pages/Forbidden";
import { NotFound } from "../pages/NotFound";
import { Dashboard3D } from "../pages/Dashboard3D";
import { AppShell } from "../shell/AppShell";
import { EmployeeDirectory } from "../workforce/employees/EmployeeDirectory";
import { EmployeeCreate } from "../workforce/employees/EmployeeCreate";
import { EmployeeProfile } from "../workforce/employees/EmployeeProfile";
import { OrgListView } from "../workforce/org/OrgListView";

/** Demo-backed modules. These render seed-backed, interactive pages in demo
 *  mode so the whole product is explorable; the real backend services replace
 *  them transparently via VITE_API_URL when available. */
const MODULE_ROUTES = [
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
  "fintech",
  "retail",
  "gov",
];

export const router = createBrowserRouter([
  {
    path: "/",
    element: <AppShell />,
    children: [
      { index: true, element: <Dashboard3D /> },
      { path: "3d", element: <Dashboard3D />,},
      { path: "workforce/employees", element: <EmployeeDirectory /> },
      { path: "workforce", element: <Navigate to="/workforce/employees" replace /> },
      { path: "workforce/employees/new", element: <EmployeeCreate /> },
      { path: "workforce/employees/:ulid", element: <EmployeeProfile /> },
      { path: "workforce/:resource", element: <OrgListView /> },
      ...MODULE_ROUTES.map((module) => ({
        path: module,
        element: <DemoModule module={module} />,
      })),
      { path: "403", element: <Forbidden /> },
      { path: "404", element: <NotFound /> },
    ],
  },
  { path: "*", element: <NotFound /> },
]);
