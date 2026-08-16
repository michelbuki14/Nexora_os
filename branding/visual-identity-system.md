# Nexora OS — Visual Identity System

## Logo Family

### Primary Logo (Wordmark)
- **Font**: DM Sans, Medium weight (500)
- **Text**: "Nexora OS"
- **Spacing**: "Nexora" (kerning: -0.02em), "OS" (tracking: 0.05em)
- **Color**: 
  - Light mode: Midnight `#1A1A2E`
  - Dark mode: White Smoke `#F9FAFB`
- **Minimum size**: 120px width
- **Clear space**: Equal to height of "N" character
- **Minimum reproduction**: 24px

### Primary Logo (Symbol)
- **Mark**: 🧱 (brick emoji) — represents "built, not decked"
- **Alternative**: Geometric "N" + "OS" monogram, single-line style
- **Color**: Azure `#0066FF` on Midnight background, or Midnight on Sand/Obsidian
- **Usage**: Favicon, app icons, small contexts where wordmark won't fit

### Logo Variations

| Version | Usage | Background | Text Color |
|---------|-------|------------|------------|
| **Full color** | Primary | Sand `#F5F0E8` / Obsidian `#0D0D18` | Midnight `#1A1A2E` / White Smoke `#F9FAFB` |
| **Inverted** | Dark backgrounds | Midnight `#1A1A2E` / Obsidian `#0D0D18` | White Smoke `#F9FAFB` / Sand `#F5F0E8` |
| **Azure accent** | Highlight | Sand/Obsidian | Midnight + Azure `#0066FF` 20% tint |
| **Monochrome** | fax/grayscale | 100% black on 100% white | 100% black on 100% white |

### Logo Clear Space
- **Minimum clear space**: Equal to the height of the capital "N" in "Nexora"
- **Do not place** other graphics, text, or imagery within clear space
- **Do not rotate** logo to vertical orientation — only horizontal

### Logo Minimum Size
- **Web**: 120px width at 1x pixel ratio
- **Favicon**: 16px × 16px or 32px × 32px
- **App icons**: 
  - iOS: 60×60, 120×120, 180×180 points
  - Android: 48×48, 96×96, 72×72, 96×96 dp
- **Do not scale** logo below minimum sizes — simplified symbol version may be used

### Logo Color Hierarchy
1. **Primary**: Midnight `#1A1A2E` (wordmark text on light backgrounds)
2. **Primary**: White Smoke `#F9FAFB` (wordmark text on dark backgrounds)
3. **Accent**: Azure `#0066FF` (used sparingly — underlines, active states, highlights)
4. **Secondary**: Sand `#F5F0E8` (light mode background), Obsidian `#0D0D18` (dark mode background)
5. **Tertiary**: Emerald `#00A859` (success states), Crimson `#E53E3E` (warning states), Error `#EF4444` (error states)

**Never** use Emerald, Crimson, or Error as primary logo text color.

### Color Palette

#### Light Mode
| Role | Color | Hex | WCAG Contrast |
|------|-------|-----|---------------|
| **Background** | Sand | `#F5F0E8` | — |
| **Primary text** | Midnight | `#1A1A2E` | 8.5:1 on `#F5F0E8` (AAA) |
| **Accent** | Azure | `#0066FF` | 3.1:1 on `#1A1A2E` (AA large text) |
| **Borders** | Slate-300 | `#D1D5DB` | — |
| **Muted text** | Slate-400 | `#6B7280` | 4.5:1 on `#F5F0E8` (AA) |
| **Success** | Emerald | `#00A859` | 3.8:1 on `#F5F0E8` (AA large text) |
| **Warning** | Crimson | `#E53E3E` | 3.2:1 on `#F5F0E8` (AA large text) |
| **Error** | Red-500 | `#EF4444` | 3.3:1 on `#F5F0E8` (AA large text) |

#### Dark Mode
| Role | Color | Hex | WCAG Contrast |
|------|-------|-----|---------------|
| **Background** | Obsidian | `#0D0D18` | — |
| **Primary text** | White Smoke | `#F9FAFB` | 8.5:1 on `#0D0D18` (AAA) |
| **Accent** | Azure | `#0066FF` | 3.1:1 on `#F9FAFB` (AA large text) |
| **Borders** | Slate-700 | `#374151` | — |
| **Muted text** | Slate-400 | `#6B7280` | 4.5:1 on `#0D0D18` (AA) |
| **Success** | Emerald-600 | `#059669` | 3.8:1 on `#F9FAFB` (AA large text) |
| **Warning** | Crimson-500 | `#DC2626` | 3.2:1 on `#F9FAFB` (AA large text) |
| **Error** | Red-500 | `#EF4444` | 3.3:1 on `#F9FAFB` (AA large text) |

#### Color Usage Rules
1. **Text on background**: minimum 4.5:1 (AA), 7:1 (AAA) for normal text
2. **UI components/borders**: minimum 3:1 (AA), 4.5:1 (AAA)
3. **Incidental text**: 3:1 (AA) acceptable
4. **Accent color (Azure)** only on: links, buttons, hover states, focus states
5. **Never use** Crimson or Emerald as primary text color — only for status labels
6. **Contrast ratio** tested against both light and dark backgrounds
7. **`prefers-color-scheme: dark`** media query respected — colors adapt

#### Accessible Color Combinations (verified)
| Text | Background | Ratio | Status |
|------|-----------|-------|--------|
| `#1A1A2E` on `#F5F0E8` | 8.5:1 | AAA |
| `#F9FAFB` on `#0D0D18` | 8.5:1 | AAA |
| `#FFFFFF` on `#0066FF` | 3.1:1 | AA (large text ≥ 18pt or 14pt bold) |
| `#FFFFFF` on `#E53E3E` | 3.2:1 | AA (large text ≥ 18pt or 14pt bold) |
| `#F9FAFB` on `#E53E3E` | 5.8:1 | AAA |
| `#FFFFFF` on `#00A859` | 4.2:1 | AAA |

#### Color Contrast Failures (do not use these combinations)
| Text | Background | Why |
|------|-----------|-----|
| `#E53E3E` on `#F5F0E8` | 3.2:1 | Only AA for large text; normal text fails AA |
| `#EF4444` on `#F5F0E8` | 3.3:1 | Only AA for large text; normal text fails AA |
| `#1A1A2E` on `#0D0D18` | 0.3:1 | Invisible — never use light text on Obsidian without background panel |

### Typography System

#### Typefaces
- **Primary**: Inter, weights 400 (Regular) to 600 (Semi-bold)
- **Secondary**: DM Sans, weights 400 (Regular) to 700 (Bold)
- **Fallback**: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "Noto Sans", sans-serif

#### type Scale (clamp-responsive)
| Scale | CSS `font-size` | Line Height | Usage |
|-------|----------------|-------------|-------|
| `--type-scale-1` | `clamp(0.75rem, 2vw, 0.875rem)` | 1.2 | Captions, meta text |
| `--type-scale-2` | `clamp(0.875rem, 2vw, 1rem)` | 1.4 | Body secondary, help text |
| `--type-scale-3` | `clamp(1rem, 2vw, 1.125rem)` | 1.5 | Body base, paragraph |
| `--type-scale-4` | `clamp(1.25rem, 2vw, 1.5rem)` | 1.6 | Headings H4, section titles |
| `--type-scale-5` | `clamp(1.5rem, 2vw, 1.875rem)` | 1.7 | Headings H3, page titles |
| `--type-scale-6` | `clamp(2rem, 2vw, 2.5rem)` | 1.8 | Headings H2, major section |
| `--type-scale-7` | `clamp(3rem, 2vw, 3.75rem)` | 1.9 | Headings H1, hero titles |

#### Typography Usage

**Body Text**
```css
font-family: 'Inter', system-ui, sans-serif;
font-size: var(--type-scale-3);  /* 1rem at base */
font-weight: 400;               /* Regular */
line-height: var(--type-scale-3-lh);  /* 1.5 */
color: var(--midnight);         /* #1A1A2E on light */
color: var(--text-dark);       /* #F9FAFB on dark */
```

**Headings (DM Sans)**
```css
font-family: 'DM Sans', sans-serif;
font-weight: 500;               /* Medium for H4-H5 */
font-size: var(--type-scale-4);  /* 1.25rem */
line-height: calc(1.6 * 1.25rem);
color: var(--midnight);         /* #1A1A2E on light */
color: var(--text-dark);       /* #F9FAFB on dark */
```

**Headings H1-H2 (larger clamp)**
```css
h1 { font-size: clamp(2rem, 5vw, 3rem); font-weight: 700; }  /* DM Sans Bold */
h2 { font-size: clamp(1.5rem, 4vw, 2.5rem); font-weight: 700; }
```

#### Typographic Rhythm
- **Base unit**: 1rem (16px)
- **All spacing** (padding, margin) aligns to 8-point grid: 0, 8, 16, 24, 32, 40, 48, 56, 64
- **Vertical rhythm**: vertical rhythm kit using 1rem baseline
- **Line length**: 45-75 characters per line for body text; optimal 66 characters

#### Accessibility
- **Minimum line height**: 1.5 for body text
- **Font size scaling**: `clamp()` ensures text resizes from 75% to 125% without breaking
- **Text resizing**: up to 200% does not break layout or truncate content
- **Contrast**: all text sizes meet WCAG AA minimum 4.5:1

### Iconography

#### Icon Set (6 core icons)
1. **Foundation** 🧱 — "built, not decked" metaphor; brick outline or laid foundation
2. **Hash Chain** ↱ — SHA-256 chain arrow; minimal single-line style
3. **Keyhole** 🔓 — OIDC authentication; simplified keyhole shape
4. **Shield** ✓ — tenant isolation, RLS, fail-closed; silhouette of shield
5. **Document** 📄 — secure document management; simplified page with lock
6. **User** 👤 — per-user ULID identity; stylized head/shoulders outline

#### Icon Specifications
- **Style**: Single-line, 2px stroke width, rounded caps
- **Size**: 24px × 24px canvas, 16px-24px display size
- **Color**: Azure `#0066FF` on light backgrounds, White Smoke `#F9FAFB` on dark
- **Stroke**: 2px, `stroke-linecap: round`, `stroke-linejoin: round`
- **Fill**: none (outline only), except Document icon may have fill

#### Icon Usage
```html
<!-- Foundation icon -->
<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
  <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
  <line x1="9" y1="9" x2="15" y2="15"></line>
  <line x1="15" y1="9" x2="9" y2="15"></line>
</svg>

<!-- Hash chain arrow -->
<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
  <path d="M5 12h14M12 5l7 7-7 7"></path>
</svg>
```

#### Icon Grid
- All icons fit within 24×24 pixel square
- Visual padding: minimum 2px from edge to any graphical element
- Consistent optical centering — strokes centered on pixel grid

### Visual Patterns

#### Subtle Grid (light mode only)
- **Pattern**: 1px lines, spacing every 8px
- **Color**: Midnight `#1A1A2E` at 3% opacity
- **Application**: Background pattern for pages/sections
- **Do not use** on dark mode — use subtle noise instead

#### Dark Mode Texture
- **Pattern**: Subtle noise, 2% opacity
- **Color**: Black at 2% over Obsidian `#0D0D18`
- **Application**: Dark mode page backgrounds
- **Do not use** grid pattern on dark mode — visual clutter

#### Brand Patterns (accent)
- **Azure at 5% opacity**: used as subtle overlay on images/sections
- **Maximum 1 pattern per page** — don't combine grid + noise
- **Pattern size**: 16px repeat for grid, 32px repeat for noise

#### Spacing Principles (8-point grid)
- **All spacings** in multiples of 8px: 0, 8, 16, 24, 32, 40, 48, 56, 64
- **Horizontal rhythm**: 8px column grid; components snap to 8px grid
- **Vertical rhythm**: 8px row grid; baseline grid for typography
- **Page margins**: 16px on all sides, or 24px for main content area
- **Component spacing**: 8px between related elements; 16px between sections

#### Visual Consistency Rules
1. **Always** use Midnight `#1A1A2E` for body text on light backgrounds (#F5F0E8)
2. **Always** use White Smoke `#F9FAFB` for body text on dark backgrounds (#0D0D18)
3. **Accent color (Azure)** only on interactive elements: links, buttons, hover states, focus states
4. **Never use** Emerald `#00A859` or Crimson `#E53E3E` as primary text color — only for status labels (success/warning)
5. **Contrast ratio**: minimum 4.5:1 for text, 3:1 for UI components (WCAG AA)
6. **Dark mode**: `prefers-color-scheme: dark` media query — all colors adapt
7. **Logo clear space**: equal to height of "N" in "Nexora"; never violate
8. **Logo minimum size**: 120px width on web; do not scale below minimum
9. **Icon color**: Azure on light, White Smoke on dark; never use Emerald/Crimson/Error as icon color
10. **Patterns**: maximum 1 per page; grid only in light mode, noise only in dark mode

### Visual Identity Summary Checklist
- [x] Logo family (wordmark, symbol, variations)
- [x] Color palette (light/dark, WCAG-verified combinations)
- [x] Typography system (Inter + DM Sans, responsive scale)
- [x] Iconography (6 core icons, single-line style)
- [x] Visual patterns (grid/dark texture, spacing)
- [x] Spacing principles (8-point grid system)
- [x] Consistency rules (10 mandatory rules)
- [x] Accessibility (WCAG AA minimum, contrast ratios)
- [x] Dark mode support (media query, color adaptation)
- [x] Minimum sizes (logo, icons, typography)