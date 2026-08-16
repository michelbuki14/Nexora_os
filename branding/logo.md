# Nexora OS Brand Guidelines

## Visual Identity

### Primary Logo Mark
```
█� █� ████ ████ ████ ████
█    █   █   █   █   █
█    █   █   █   █   █
 ████    █   █   █   █
  █       █   █   █   █
  █        █    ████
```

### Wordmark
**Nexora OS**

- Typeface: Inter (or system UI font)
- Color: `#1A1A2E` (deep midnight blue) on light backgrounds
- Color: `FFFFFF` (white) on dark backgrounds
- Weight: Regular for "Nexora", Medium for "OS"

### Color Palette
| Color | Hex | Usage |
|-------|-----|-------|
| Midnight | `#1A1A2E` | Primary, headings, main text |
| Azure | `#0066FF` | Accents, links, highlights |
| Sand | `#F5F0E8` | Backgrounds, cards |
| Obsidian | `#0D0D18` | Dark mode, footers |
| Emerald | `#00A859` | Success states, positive indicators |
| Crimson | `#E53E3E` | Error states, warnings |

### Typography
- **Primary**: Inter, 400-600 weight
- **Secondary**: DM Sans, 400-500 weight (for display/headlines)
- **Scale**: 1rem = 16px base, modular scale 1.25x/1.5x

## Voice & Tone

### Writing Style
- **Direct, no-nonsense**: Say exactly what it is, no hype
- **Outcome-focused**: What breaks for users if... not implementation details
- **Short sentences**: One idea per sentence
- **No AI vocabulary**: No "delve", "robust", "comprehensive", "nuanced"
- **Plain language**: "you can now..." not "refactored the..."

### Prohibited Phrases
- "delve into"
- "the bottom line is"
- "here's the kicker"
- "in today's landscape"
- "it's worth noting that"

### Allowed Constructions
- "this means"
- "users will"
- "the system does"
- "here's what happens"

## Core Messaging

### Tagline
> **Honest infrastructure for African commerce — built, not decked.**

### Positioning Statement
Nexora OS is infrastructure for African commerce, providing tenant isolation, security features, and compliance capabilities for B2B SaaS. The product addresses regulated industries (fintech, payment service providers, savings cooperatives) with a SaaS model at $2-5K/tenant/month. The technical foundation is solid with proper CI/CD pipelines and appropriate test coverage.

### Key Proof Points (Live, Demonstratable)
- **Tamper-evident by construction**: SHA-256 chained audit events; verify-hash-chain endpoint replays the full chain and flags tampering
- **Tenant isolation enforced in the database**: PostgreSQL Row-Level Security across audit table and all workforce tables, fail-closed
- **OIDC-authenticated end to end**: Real Keycloak JWT, JWKS signature validation; live token returns 201 audit event

## Brand Assets Checklist
- [ ] Logo variants (primary, wordmark, icon-only)
- [ ] Color palette system
- [ ] Typography system
- [ ] Voice and tone guidelines
- [ ] Messaging framework
- [ ] Application of brand to UI components
- [ ] Brand documentation

## Usage Rules
1. Never use AI vocabulary (delve, robust, comprehensive, nuanced, etc.)
2. Always lead with user outcomes, not implementation details
3. Keep sentences short; one idea per sentence
4. Maintain contrast ratios: minimum 4.5:1 for text, 3:1 for UI components
5. Dark mode: use Obsidian (#0D0D18) as primary dark background, Azure (#0066FF) as accent
6. Light mode: use Sand (#F5F0E8) as primary background, Midnight (#1A1A2E) as text
7. Never reference internal version numbers or branch states in public-facing material