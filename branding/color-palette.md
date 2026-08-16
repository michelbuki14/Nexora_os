# Nexora OS Color Palette

## Light Mode
- **Background**: `#F5F0E8` (Sand)
- **Surface**: `#FFFFFF` (White)
- **Primary Text**: `#1A1A2E` (Midnight)
- **Secondary Text**: `#4A4A5A` (Muted Midnight)
- **Border**: `#D1D5DB` (Light Gray)
- **Accent**: `#0066FF` (Azure)
- **Success**: `#00A859` (Emerald)
- **Warning**: `#E53E3E` (Crimson)
- **Error**: `#EF4444` (Red-500)

## Dark Mode
- **Background**: `#0D0D18` (Obsidian)
- **Surface**: `#1F2937` (Slate-900)
- **Primary Text**: `#F9FAFB` (White Smoke)
- **Secondary Text**: `#6B7280` (Slate-400)
- **Border**: `#374151` (Slate-700)
- **Accent**: `#0066FF` (Azure)
- **Success**: `#059669` (Emerald-600)
- **Warning**: `#DC2626` (Crimson-500)
- **Error**: `#EF4444` (Red-500)

## Usage Guidelines

### Contrast Ratios
- Text on background: minimum 4.5:1 (AA), 7:1 (AAA)
- UI components/borders: minimum 3:1 (AA), 4.5:1 (AAA)
- Incidental text: 3:1 (AA) acceptable

### Accessible Combinations
| Text Color | Background Color | Ratio | Status |
|------------|-----------------|-------|--------|
| `#1A1A2E` on `#F5F0E8` | 8.5:1 | AAA |
| `#F9FAFB` on `#0D0D18` | 8.5:1 | AAA |
| `#FFFFFF` on `#0066FF` | 3.1:1 | AA (large text) |
| `#FFFFFF` on `#E53E3E` | 3.2:1 | AA (large text) |

### Component Guidelines
- **Buttons**: Azure background (`#0066FF`), white text (`#FFFFFF`), hover state 20% darker
- **Cards**: White (`#FFFFFF`) on light, Slate-900 (`#1F2937`) on dark, with border
- **Input fields**: White (`#FFFFFF`) background, Midnight (`#1A1A2E`) text, border: 1px Slate-300 (`#D1D5DB`) light, Slate-500 (`#6B7280`) dark
- **Alerts**: Use Emerald for success, Crimson for warning, Red-500 for error
- **Badges**: Azure background with white text for primary, Emerald for success, Crimson for warnings

### Dark Mode Specific
- Prefer `prefers-color-scheme: dark` media queries
- Use `safe-area-inset` for mobile browsers
- Ensure minimum touch target: 44px × 44px
- Avoid pure black (#000000) as background; use Obsidian (#0D0D18) instead

### Responsive Considerations
- Mobile-first approach: single column layout on <640px
- Scale typography: clamp(1rem, 2vw, 1.25rem) for body text
- Minimum touch targets: 44px × 44px
- Horizontal scroll only for data tables, not layout