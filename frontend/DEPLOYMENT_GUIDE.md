# Nexora OS 3D Frontend - Vercel Deployment

## One-Click Deploy

### Option 1: Vercel Dashboard (Easiest)
1. Go to [vercel.com/new](https://vercel.com/new)
2. Import this GitHub repository
3. Vercel auto-detects:
   - Framework: Vite
   - Build Command: `bun run build`
   - Output Directory: `dist`
4. Click **Deploy**

### Option 2: Vercel CLI
```bash
cd frontend
npx vercel --prod
# Or: npx vercel --prod --dist dist
```

### Option 3: GitHub Auto-Deploy
1. Push to GitHub
2. Go to vercel.com → Settings → GitHub Integration
3. Connect repository → Enable Automatic Deploys

## Why Vercel Works for This App

| Feature | Why It Matters |
|---------|----------------|
| **Vite-first** | Zero config — Vercel auto-detects Vite |
| **Edge network** | Three.js renders fast globally |
| **Automatic HTTPS** | Free SSL for custom domains |
| **SPA Routing** | `vercel.json` handles client-side routing |
| **Unlimited bandwidth** | Three.js assets + WebGL content |
| **Preview deployments** | Share `git PR` URLs for testing |

## Post-Deploy

```bash
# After deploy, visit your Vercel URL
# The 3D landing page loads at /
# Works in Chrome/Firefox/Safari with WebGL 2 support
```

## Troubleshooting

| Issue | Fix |
|-------|-----|
| Blank page | Check browser console for WebGL errors |
| Missing textures | Ensure `dist/assets/` uploaded correctly |
| 404 on refresh | `vercel.json` handles SPA routing |
| Slow load | Enable CDN caching on Vercel dashboard |

## Files Modified for Deployment

- `vercel.json` — New: Vercel build config
- `frontend/src/app/routes.tsx` — Removed unused Dashboard import
- `frontend/src/pages/Dashboard3D.tsx` — Cleaned unused imports
- `frontend/src/components/ConfettiCelebration.tsx` — Fixed Particle type
- `frontend/src/components/DesignCanvas.tsx` — Fixed camera position typing

## Build Verification

Run locally before deploying:
```bash
cd frontend
bun install    # install deps
bun run build  # generate dist/
bun run preview  # test locally at http://localhost:4173
```

Then deploy to Vercel. The 3D landing page with workforce bar chart, payroll timeline, and tenant donut will be live at your Vercel URL.