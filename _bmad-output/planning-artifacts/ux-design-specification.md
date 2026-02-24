---
stepsCompleted:
  - step-01-init
inputDocuments:
  - path: _bmad-output/planning-artifacts/prd.md
    type: prd
    description: Primary Product Requirements Document
    included: true
  - path: _bmad-output/brainstorming/brainstorming-session-2026-02-18.md
    type: brainstorming
    description: Initial product/architecture brainstorming session
    included: true
  - path: _bmad-output/planning-artifacts/architecture.md
    type: architecture
    description: Architecture Decision Document
    included: true
workflowType: 'ux-design'
project_name: 'skaild2'
user_name: 'mathieu'
date: '2026-02-18'
---

# UX Design Specification skaild2

### Tailwind Configuration Mapping (Mermaidcore)

Use this as a starting point in `tailwind.config.{js,ts}`:

```js
// tailwind.config.js (excerpt)
module.exports = {
  darkMode: ['class', '[data-theme="mermaidcore-dark"]'],
  theme: {
    extend: {
      colors: {
        ocean: {
          deep: '#020617',
          deeper: '#04101F',
        },
        surface: {
          glass: 'rgba(15, 23, 42, 0.55)',
        },
        accent: {
          aqua: '#4FD1C5',
          teal: '#14B8A6',
          lilac: '#A855F7',
          silver: '#E5E7EB',
        },
        text: {
          primary: '#F9FAFB',
          muted: '#9CA3AF',
        },
        status: {
          danger: '#FB7185',
          warn: '#FBBF24',
        },
      },
      borderRadius: {
        'mc-button': '999px',
        'mc-card': '18px',
      },
      boxShadow: {
        'mc-primary-glow': '0 0 24px rgba(79, 209, 197, 0.35)',
      },
      backgroundImage: {
        'mc-primary': 'linear-gradient(135deg, #4FD1C5 0%, #A855F7 60%, #E5E7EB 100%)',
        'mc-ocean': 'linear-gradient(to bottom, #020617, #04101F)',
      },
    },
  },
  plugins: [],
};
```

Example usage:

```html
<main class="min-h-screen bg-mc-ocean text-text-primary">
  <section class="mc-surface shadow-mc-primary-glow p-6">
    <button
      class="inline-flex items-center px-5 py-2 rounded-mc-button bg-mc-primary text-text-primary shadow-mc-primary-glow"
    >
      Add Route
    </button>
  </section>
</main>
```
**Author:** mathieu
**Date:** 2026-02-18

---

<!-- UX design content will be appended sequentially through collaborative workflow steps -->

## Visual Language: Mermaidcore

- Mode: default dark, calm/professional with a subtle magical shimmer.
- Background: custom branded background image (`skaild-background.png`) with no overlay, displayed as cover with fixed attachment for depth.
- Surfaces: glassmorphic panels with transparency and strong blur, floating over the background image.
- Accents: iridescent aqua, soft teal, pearlescent purple and silver used in gradients and outlines.
- Typography: 
  - Page titles and branding: dark text (#1E293B) for strong contrast
  - Navigation: white text on dark slate backgrounds for consistent visibility
  - Card content: lighter slate tones (slate-200/300) for improved readability

### Design Tokens (v1)

- Colors
  - `bg-ocean-deep`: Custom background image (skaild-background.png)
  - `surface-glass`: rgba(15, 23, 42, 0.55) with backdrop blur
  - `accent-aqua`: #4FD1C5
  - `accent-teal`: #14B8A6
  - `accent-lilac`: #A855F7
  - `accent-silver`: #E5E7EB
  - `text-primary`: #F9FAFB (white/bright)
  - `text-dark`: #1E293B (dark slate for titles)
  - `text-muted`: #9CA3AF
  - `text-light`: #E2E8F0 (slate-200)
  - `text-card`: #CBD5E1 (slate-300)
  - `nav-bg`: rgba(30, 41, 55, 0.8) (slate-800/80)
  - `nav-bg-hover`: rgba(51, 65, 85, 0.8) (slate-700/80)
  - `danger-coral`: #FB7185
  - `warn-amber`: #FBBF24

- Gradients
  - `gradient-primary`: linear 135deg, accent-aqua → accent-lilac → accent-silver

- Effects
  - `glass-panel`: backdrop-filter blur(18px), border 1px solid rgba(148, 163, 184, 0.5)
  - `primary-glow`: 0 0 24px rgba(79, 209, 197, 0.35)
  - `focus-ring`: 0 0 0 2px accent-aqua

### CSS Variables (Mermaidcore, Dark Mode)

Use these as a base theme, e.g. in `:root` or `[data-theme="dark"]`:

```css
:root[data-theme="mermaidcore-dark"] {
  /* Base background */
  --color-bg-ocean-deep-top: #020617;
  --color-bg-ocean-deep-bottom: #04101F;

  /* Surfaces */
  --color-surface-glass: rgba(15, 23, 42, 0.55);
  --border-surface-glass: 1px solid rgba(148, 163, 184, 0.5);
  --blur-surface-glass: 18px;

  /* Accents */
  --color-accent-aqua: #4FD1C5;
  --color-accent-teal: #14B8A6;
  --color-accent-lilac: #A855F7;
  --color-accent-silver: #E5E7EB;

  /* Text */
  --color-text-primary: #F9FAFB;
  --color-text-muted: #9CA3AF;
  --color-text-dark: #1E293B;
  --color-text-light: #E2E8F0;
  --color-text-nav: #CBD5E1;

  /* Status */
  --color-danger-coral: #FB7185;
  --color-warn-amber: #FBBF24;

  /* Gradients */
  --gradient-primary: linear-gradient(135deg, #4FD1C5 0%, #A855F7 60%, #E5E7EB 100%);

  /* Effects */
  --shadow-primary-glow: 0 0 24px rgba(79, 209, 197, 0.35);
  --ring-focus: 0 0 0 2px #4FD1C5;

  /* Radii */
  --radius-button: 999px;
  --radius-card: 18px;

  /* Spacing */
  --space-card-padding: 1.25rem;
  --space-section-gap: 1.5rem;
}

.mc-surface {
  background: var(--color-surface-glass);
  backdrop-filter: blur(var(--blur-surface-glass));
  border: var(--border-surface-glass);
  border-radius: var(--radius-card);
}

.mc-button-primary {
  border-radius: var(--radius-button);
  background-image: var(--gradient-primary);
  color: var(--color-text-primary);
  box-shadow: var(--shadow-primary-glow);
}

.mc-button-secondary {
  border-radius: var(--radius-button);
  background-color: rgba(15, 118, 110, 0.18);
  border: 1px solid rgba(20, 184, 166, 0.7);
  color: var(--color-text-primary);
}
```

### Components (High Level)

- Primary Button
  - Shape: rounded (8–999px radius depending on control size).
  - Fill: gradient-primary with medium opacity over background.
  - Chrome: 1px silver border, subtle inner highlight.
  - Hover: slightly higher opacity, small scale-up, primary-glow, light “shimmer” across the gradient.

- Secondary Button
  - Shape: same radius as primary.
  - Fill: mostly transparent with soft teal tint.
  - Chrome: 1px teal border, no strong glow.
  - Hover: color shift toward lighter aqua, minimal glow.

- Surface / Card
  - Background: glass-panel token (transparent + blur).
  - Layout: floating tiles with comfortable padding and rounded corners.
  - Borders: subtle 1px silver/blue edge; optional soft inner shadow to feel like layered glass.

### Key Screen: Admin Dashboard (Concept)

- Background layer: full viewport with custom background image (`skaild-background.png`), no overlay gradient to preserve image clarity
  - Background styling: `background-size: cover`, `background-position: center`, `background-attachment: fixed` for parallax effect
- Top bar: translucent glass strip with logo/wordmark on the left, compact status indicators and user menu on the right
  - Logo/titles: dark text (`#1E293B`) without background boxes for clean appearance
- Left rail: slim navigation with icon + label for core areas (Dashboard, Routes, Identity, Certificates, Settings)
  - Navigation items: white text on dark slate-800/80 background boxes (always visible)
  - Hover state: slightly lighter background (slate-700/80) with dimmer text (slate-300)
  - Uses aqua/silver line icons
- Main content: central glass panels for "Quick status" (health of proxies, certs, SSO), "Recent changes", and "Active sessions / sign-ins"
  - Card headings: bright slate-200 text for visibility
  - Card body text: slate-300 for improved readability against background
  - Glass panels maintain original glassmorphic effect with backdrop blur
- Primary CTAs: Mermaidcore primary buttons for "Add Route", "Connect IdP", and "Issue Certificate", placed in prominent but not overwhelming positions
- Feedback: warnings use warn-amber on glass panels; errors use danger-coral, always legible against dark mode

