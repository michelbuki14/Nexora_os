# Nexora OS Frontend

Next.js 15 + React 18 + Tailwind CSS 3 frontend for the Nexora OS platform.

## Current status

The frontend is in early development. The backend services (workforce, payroll) are complete, but frontend pages are not yet built.

## Backend services available

| Service | Base URL | Endpoints |
|---------|----------|-----------|
| API Gateway | `http://localhost:3001` | `/health`, `/api/v1` |
| Workforce | `http://localhost:3002` | 100+ endpoints for employees, contracts, compensation, attendance, leave |
| Payroll | `http://localhost:3003` | 19 endpoints for payroll runs, payslips, components, configs |

## Tech stack

- Next.js 15 (App Router)
- React 18
- Tailwind CSS 3
- Three.js / @react-three/fiber (3D visuals — Phase 4)
- Prisma + SQLite (local dev)
- Supabase (production)

## Local development

```bash
cd frontend
npm install
npm run dev
```

The frontend connects to backend services running on ports 3001-3003.

## Pages to build (Phase 2-3)

- Employee list + detail
- Contract + compensation management
- Attendance + leave request
- Payroll run list + detail + preview
- Payslip viewer + download
- Approval workflow UI
- Employee self-service portal (my payslips, payroll info)

## 3D Visualizations (Phase 4)

- Financial data visualization with Three.js
- Interactive charts for payroll analytics
- Dashboard with real-time metrics
