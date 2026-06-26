---
name: design-system-toolkit
description: Unified design system toolkit — design process & taste (claude-design), formal token specs (DESIGN.md), 54 real-world design systems (popular-web-designs), hand-drawn diagrams (excalidraw), and architecture diagrams (architecture-diagram).
version: 2.0.0
tags: [design, design-system, tokens, ui, ux, html, prototype, deck, html-artifact, diagram, svg, excalidraw, architecture]
related_skills: []
---

# Design System Toolkit

Complete design capabilities for Hermes agents. Combines design process/taste, formal token specification, real-world design system references, hand-drawn diagrams, and technical architecture diagrams into one unified skill.

## Quick Navigation

| Sub-Skill | Purpose | When to Use |
|-----------|---------|-------------|
| [Design Process](#1-design-process--taste) | Scoping, context gathering, variant exploration, anti-slop rules | From-scratch HTML artifacts: landing pages, prototypes, decks, component labs |
| [Token Specs (DESIGN.md)](#2-designmd-token-specs) | Author/validate/export Google's DESIGN.md format | Formal, persistent design-system spec files consumed by agents over time |
| [Design System Catalog](#3-design-system-catalog-54-systems) | 54 ready-to-paste design systems (Stripe, Linear, Vercel, etc.) | "Make it look like Stripe/Linear/Vercel" — visual vocabulary for known brands |
| [Hand-Drawn Diagrams](#4-hand-drawn-diagrams-excalidraw) | Excalidraw JSON for architecture, flow, sequence diagrams | Hand-drawn aesthetic for docs, presentations, collaborative sketching |
| [Architecture Diagrams](#5-architecture-diagrams-svg) | Dark-themed SVG diagrams of software systems/cloud infra | Software architecture, cloud topology, microservice maps, security groups |

---

## 1. Design Process & Taste

**Core philosophy**: Start from context, not vibes. Avoid AI design slop.

### Workflow

1. **Understand the brief** — What, who, artifact format, locked constraints?
2. **Gather context** — Brand docs, screenshots, repo components, tokens, UI kits, copy docs
3. **Define the design system for this artifact** — Colors, type, spacing, radii, shadows, motion, components, interactions
4. **Choose format** — Static comparison (side-by-side HTML), clickable prototype, fixed-size deck, component lab, motion study
5. **Build artifact** — Single self-contained HTML file (default), preserve versions for revisions
6. **Verify** — File exists, syntax checks, browser console errors, screenshot inspection
7. **Report** — Exact path, what was created, caveats, next decision

### Anti-Slop Rules (Mandatory)

| ❌ Avoid | ✅ Prefer |
|----------|-----------|
| Aggressive gradient backgrounds | Intentional color systems |
| Glassmorphism by default | Purposeful surface treatment |
| Emoji unless brand uses them | Typography as hierarchy |
| Generic SaaS cards with icons | Content that earns its place |
| Left-border accent callout cards | Density without clutter |
| Fake dashboards with arbitrary numbers | Real data or clean placeholders |
| Stock-photo hero sections | Typography, layout, abstract texture |
| Oversized rounded rectangles as hierarchy substitute | Scale, whitespace, alignment |
| Rainbow palettes | One primary accent |
| Vague labels ("Insights", "Growth", "Scale") | Specific, content-driven labels |
| Decorative SVG pretending to be product imagery | Real supplied imagery or clean placeholders |

### Variation Rules

Default to **at least three options**:
1. **Conservative** — closest to existing patterns, lowest risk
2. **Strong-fit** — best interpretation of the brief
3. **Divergent** — more novel, discovers taste boundaries

Variations explore: layout, hierarchy, type scale, density, color posture, surface treatment, motion, interaction model, copy structure, component shape.

### HTML/CSS/JS Standards

- CSS variables for tokens, CSS grid for layout, container queries
- `text-wrap: pretty`, real focus/hover states, `prefers-reduced-motion`
- Responsive scaling, semantic HTML
- Mobile hit targets ≥ 44px
- Deck text ≥ 24px at 1920×1080
- React only when state/interaction complexity warrants it

---

## 2. DESIGN.md Token Specs

**DESIGN.md** = Google's open spec (Apache-2.0) for describing visual identity to coding agents.

### File Anatomy

```markdown
---
version: alpha
name: Heritage
description: Architectural minimalism meets journalistic gravitas.
colors:
  primary: "#1A1C1E"
  secondary: "#6C7278"
  tertiary: "#B8422E"
  neutral: "#F7F5F2"
typography:
  h1:
    fontFamily: Public Sans
    fontSize: 3rem
    fontWeight: 700
    lineHeight: 1.1
    letterSpacing: "-0.02em"
  body-md:
    fontFamily: Public Sans
    fontSize: 1rem
rounded:
  sm: 4px
  md: 8px
  lg: 16px
spacing:
  sm: 8px
  md: 16px
  lg: 24px
components:
  button-primary:
    backgroundColor: "{colors.tertiary}"
    textColor: "#FFFFFF"
    rounded: "{rounded.sm}"
    padding: 12px
  button-primary-hover:
    backgroundColor: "{colors.primary}"
---

## Overview
Architectural Minimalism meets Journalistic Gravitas...

## Colors
- **Primary (#1A1C1E):** Deep ink for headlines and core text.
- **Tertiary (#B8422E):** "Boston Clay" — sole driver for interaction.

## Typography
Public Sans for everything except small all-caps labels...

## Components
`button-primary` is the only high-emphasis action on a page...
```

### Token Types

| Type | Format | Example |
|------|--------|---------|
| Color | `#` + hex (sRGB) | `"#1A1C1E"` |
| Dimension | number + unit (`px`, `em`, `rem`) | `48px`, `-0.02em` |
| Token reference | `{path.to.token}` | `{colors.primary}` |
| Typography | object with fontFamily, fontSize, fontWeight, lineHeight, letterSpacing, fontFeature, fontVariation | see above |

### Canonical Section Order (Enforced)

1. Overview (alias: Brand & Style)
2. Colors
3. Typography
4. Layout (alias: Layout & Spacing)
5. Elevation & Depth (alias: Elevation)
6. Shapes
7. Components
8. Do's and Don'ts

### CLI Workflow (`npx @google/design.md`)

```bash
# Validate structure + token refs + WCAG contrast
npx -y @google/design.md lint DESIGN.md

# Compare versions, fail on regression
npx -y @google/design.md diff DESIGN.md DESIGN-v2.md

# Export to Tailwind theme JSON
npx -y @google/design.md export --format tailwind DESIGN.md > tailwind.theme.json

# Export to W3C DTCG JSON
npx -y @google/design.md export --format dtcg DESIGN.md > tokens.json

# Print spec for agent injection
npx -y @google/design.md spec --rules-only --format json
```

### Pitfalls

- **Don't nest component variants** — `button-primary.hover` wrong, `button-primary-hover` sibling key right
- **Hex colors must be quoted strings** — YAML chokes on `#`
- **Negative dimensions need quotes** — `letterSpacing: "-0.02em"`
- **Section order is enforced** — reorder user prose to canonical list
- **Token references resolve by dotted path** — `{colors.primary}` works, `{primary}` doesn't

---

## 3. Design System Catalog (54 Systems)

54 real-world design systems ready for HTML/CSS generation. Each captures complete visual language: color palette, typography, components, spacing, shadows, responsive behavior, agent prompts with exact CSS values.

### Categories

| Category | Systems |
|----------|---------|
| **AI & ML** | claude, cohere, elevenlabs, minimax, mistral.ai, ollama, opencode.ai, replicate, runwayml, together.ai, voltagent, x.ai |
| **Developer Tools** | cursor, expo, linear.app, lovable, mintlify, posthog, raycast, resend, sentry, supabase, superhuman, vercel, warp, zapier |
| **Infrastructure & Cloud** | clickhouse, composio, hashicorp, mongodb, sanity, stripe |
| **Design & Productivity** | airtable, cal, clay, figma, framer, intercom, miro, notion, pinterest, webflow |
| **Fintech & Crypto** | coinbase, kraken, revolut, wise |
| **Enterprise & Consumer** | airbnb, apple, bmw, ibm, nvidia, spacex, spotify, uber |

### Usage

```bash
# 1. Pick a design from catalog
# 2. Load template: skill_view(name="design-system-toolkit", file_path="templates/<site>.md")
# 3. Use tokens and component specs when generating HTML
# 4. Pair with generative-widgets skill to serve via cloudflared tunnel
# 5. Verify with browser_vision
```

### HTML Generation Pattern

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Page Title</title>
  <!-- Paste Google Fonts <link> from template's Hermes notes -->
  <link href="https://fonts.googleapis.com/css2?family=..." rel="stylesheet">
  <style>
    :root {
      --color-bg: #ffffff;
      --color-text: #171717;
      --color-accent: #533afd;
      /* ... more from template Section 2 */
    }
    body {
      font-family: 'Inter', system-ui, sans-serif;
      color: var(--color-text);
      background: var(--color-bg);
    }
    /* Apply component styles from template Section 4 */
    /* Apply layout from template Section 5 */
    /* Apply shadows from template Section 6 */
  </style>
</head>
<body>
  <!-- Build using component specs from the template -->
</body>
</html>
```

### Font Substitution Reference

| Proprietary Font | CDN Substitute | Character |
|------------------|----------------|-----------|
| Geist / Geist Sans | Geist (Google Fonts) | Geometric, compressed tracking |
| Geist Mono | Geist Mono (Google Fonts) | Clean monospace, ligatures |
| sohne-var (Stripe) | Source Sans 3 | Light weight elegance |
| Berkeley Mono | JetBrains Mono | Technical monospace |
| Airbnb Cereal VF | DM Sans | Rounded, friendly geometric |
| Circular (Spotify) | DM Sans | Geometric, warm |
| figmaSans | Inter | Clean humanist |
| Pin Sans (Pinterest) | DM Sans | Friendly, rounded |
| Coinbase/Sans | DM Sans | Geometric, trustworthy |
| UberMove | DM Sans | Bold, tight |
| HashiCorp Sans | Inter | Enterprise, neutral |
| waldenburgNormal (Sanity) | Space Grotesk | Geometric, slightly condensed |
| IBM Plex Sans/Mono | IBM Plex Sans/Mono | Available on Google Fonts |
| Rubik (Sentry) | Rubik | Available on Google Fonts |

When substitute is used (DM Sans for Circular, Source Sans 3 for sohne-var), follow template's weight, size, letter-spacing closely — those carry more visual identity than font face.

---

## 4. Hand-Drawn Diagrams (Excalidraw)

Generate Excalidraw JSON for hand-drawn style diagrams: architecture, flow, sequence.

### Use When

- Documentation diagrams needing human/sketch aesthetic
- Collaborative whiteboarding outputs
- Presentations where polished SVG feels too corporate
- Quick architecture sketches for team discussion

### Output Format

Excalidraw JSON with elements: rectangle, diamond, ellipse, arrow, line, text, freestyle. Semantic styling via `strokeColor`, `backgroundColor`, `strokeWidth`, `roughness`, `seed`.

### Integration

- Load JSON directly in Excalidraw web app (excalidraw.com)
- Embed in markdown via excalidraw plugin
- Export as PNG/SVG for static docs

---

## 5. Architecture Diagrams (SVG)

Generate dark-themed SVG diagrams of software systems and cloud infrastructure as standalone HTML files with inline SVG graphics.

### Visual Language

| Color | Semantic Meaning |
|-------|------------------|
| Cyan | Frontend |
| Emerald | Backend |
| Violet | Database |
| Amber | Cloud/AWS |
| Rose | Security |
| Orange | Message Bus |

- Font: JetBrains Mono
- Background: Grid pattern
- Best for: software architecture, cloud/VPC topology, microservice maps, service-mesh diagrams, database + API layer diagrams, security groups, message buses

### When to Use vs Excalidraw

| Use Architecture Diagram | Use Excalidraw |
|--------------------------|----------------|
| Technical infra deck, dark aesthetic | Hand-drawn/sketch aesthetic needed |
| Precise component positioning | Collaborative/iterative sketching |
| Semantic color coding required | Softer, less formal presentation |
| Standalone HTML artifact needed | Embed in Excalidraw-native workflows |

### Output

Single self-contained HTML file with inline SVG. No external dependencies.

---

## Decision Matrix: Which Sub-Skill?

| User Asks For... | Use Sub-Skill |
|------------------|---------------|
| "Design a landing page / prototype / deck" | Design Process |
| "Create a DESIGN.md file / token spec" | DESIGN.md Token Specs |
| "Make it look like Stripe / Linear / Vercel" | Design System Catalog |
| "Draw a hand-sketch architecture diagram" | Hand-Drawn Diagrams |
| "Generate a dark SVG architecture diagram" | Architecture Diagrams |
| "Design a component library / design system preview" | Design Process + DESIGN.md |
| "Redesign based on screenshot / repo / brand docs" | Design Process (gather context first) |

**These compose**: Use Design System Catalog for visual vocabulary, Design Process for turning brief into artifact, DESIGN.md when output is the token file itself.

---

## Reference Files

| File | Purpose |
|------|---------|
| `templates/starter.md` | DESIGN.md starter template |
| `templates/*.md` (54 files) | Design system templates (stripe.md, linear.app.md, vercel.md, etc.) |
| `references/hub-discovery.md` | URL-only Hugging Face workflows (from llama-cpp) |

---

## When to Use This Skill

- Any design task: landing pages, prototypes, decks, component labs, motion studies
- Formal design token specification (DESIGN.md authoring, validation, export)
- Brand-matched UI generation (54 real design systems)
- Architecture/system diagrams (SVG or hand-drawn)
- Design system previews and component explorations