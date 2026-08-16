# Nexora OS Typography System

## Typefaces

### Primary: Inter
- **Weight range**: 400 (Regular) to 600 (Semi-bold)
- **Available weights**: 400, 500, 600
- **Character set**: Latin Extended, Greek, Cyrillic
- **Why Inter**: Readable at small sizes, distinct letterforms (l vs I vs 1, o vs 0), excellent web performance

### Secondary: DM Sans
- **Weight range**: 400 (Regular) to 700 (Bold)
- **Available weights**: 400, 500, 600, 700
- **Character set**: Latin Extended
- **Why DM Sans**: Geometric, friendly, good for headlines and display; pairs well with Inter

## Scale

| Scale | Size | Line Height | Usage |
|-------|------|-------------|-------|
| `--type-scale-1` | 0.75rem (12px) | 1.2 | Captions, meta text |
| `--type-scale-2` | 0.875rem (14px) | 1.4 | Body secondary, help text |
| `--type-scale-3` | 1rem (16px) | 1.5 | Body base, paragraph |
| `--type-scale-4` | 1.25rem (20px) | 1.6 | Headings H4, section titles |
| `--type-scale-5` | 1.5rem (24px) | 1.7 | Headings H3, page titles |
| `--type-scale-6` | 2rem (32px) | 1.8 | Headings H2, major section |
| `--type-scale-7` | 3rem (48px) | 1.9 | Headings H1, hero titles |

## Usage Guidelines

### Text Sizes in CSS
```css
/* Body copy */
font-size: var(--type-scale-3);  /* 1rem */
line-height: var(--type-scale-3-lh);  /* 1.5 */

/* Headings */
font-size: var(--type-scale-4);  /* 1.25rem */
font-weight: 600;
line-height: calc(1.6 * 1.25rem);

/* Captions */
font-size: var(--type-scale-1);
font-size: 0.75rem;
line-height: 1.2;
color: var(--color-secondary-text);
```

### Vertical Rhythm
- Base unit: 1rem (16px)
- All spacing and typography should align to this rhythm
- Components should have padding/margin that's a multiple of 0.25rem

### Accessibility
- Minimum line height: 1.5 for body text
- Font size scaling: use `clamp()` or `vw` units with fallbacks
- Ensure text resizing up to 200% doesn't break layout
- Contrast ratios apply to all text sizes

### Responsive typography
```css
/* Scale text based on viewport */
h1 {
  font-size: clamp(2rem, 5vw, 3rem);
}

/* Never go below minimum or above maximum */
p {
  font-size: clamp(0.875rem, 2vw, 1rem);
}
```

## Brand-Specific Applications

### Headings (DM Sans)
- Use Medium (500) or Bold (700) weight
- All caps for primary navigation labels
- Tracked kerning: `-0.02em` for large headings

### Body Text (Inter)
- Regular (400) weight for paragraph text
- Medium (500) for emphasized sentences
- No all-caps for body text (use Inter's built-in small caps if needed)

### UI Controls
- Button text: Inter Medium (500), tracking: `0.05em`
- Input placeholders: Inter Regular (400), faded to 40% opacity
- Error messages: Inter Regular (400), Crimson color