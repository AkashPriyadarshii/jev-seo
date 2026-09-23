# DESIGN.md - jev-seo Launch Telemetry & Motion Specification

## 1. Design Read & Vibe Calibration
Reading this as: Developer launch video composition for Rust engineers and AI coding agent builders, with raw high-density terminal authority and tactile mechanical speed, leaning toward Industrial Brutalist Terminal plus HUD Telemetry.

### Style Dials
- VARIANCE: 8 / 10 (distinctive asymmetrical layout, technical telemetry readouts, terminal scanlines)
- MOTION: 8 / 10 (ODE RK4 spring kinematics, snap curves, realistic typing cadence)
- DENSITY: 9 / 10 (compact benchmark metrics, R01-R50 rule chips, AI bot permissions table)

---

## 2. Design Contract & 5-Second Gate
- First-Read Object: The live BFS crawl telemetry terminal displaying sub-second latency (840ms), Grade A+ 94/100, and $0 cost differential against Semrush ($139/mo).
- Primary Action: Developer runs `cargo install jev-seo`.
- First Read Clarity: Within 1.5 seconds, the viewer immediately grasps that jev-seo replaces $139/mo bloated SaaS suites with a blazing fast local Rust CLI that consumes under 30MB RAM.

---

## 3. Substrate & OKLCH Color Tokens
Locked Substrate: Deep Carbon Obsidian (strictly dark mode, zero substrate mixing).

- Surface Base (Substrate): `oklch(0.14 0.02 250)` (#0b0f14 tinted toward deep obsidian blue)
- Surface Elevated: `oklch(0.18 0.03 250)` (#121926)
- Surface Border: `oklch(0.26 0.04 250)` (#1e293b)
- Ink Primary: `oklch(0.98 0.01 250)` (#f8fafc)
- Ink Secondary (Refined Muted): `oklch(0.72 0.04 250)` (#94a3b8)
- Ink Inverse / Refined Dark: `#2A2A2A` (strictly no pure #000 ink)
- Accent Electric Royal: `oklch(0.60 0.22 255)` (#2563eb)
- Terminal Neon Blue: `oklch(0.75 0.16 230)` (#38bdf8)
- Signal Pass (Emerald): `oklch(0.72 0.18 155)` (#10b981)
- Signal Warning / Strike (Crimson): `oklch(0.65 0.22 25)` (#ef4444)

---

## 4. Typography Hierarchy (2+1 Ceiling)
- Display / Headline: `system-ui, -apple-system, sans-serif` (weight 800, tracking -0.03em)
- Technical Data & Code: `'SF Mono', 'JetBrains Mono', Consolas, monospace` (weight 500/700, tracking 0.01em)
- Body Copy: `system-ui, -apple-system, sans-serif` (weight 500, line-height 1.4)
- Ceiling: Strictly 2 type families (System Neo-Grotesque plus Monospace). Zero extra font faces.

---

## 5. Concentric Radius Geometry
Outer cards and inner badges follow the concentric radius formula:
$R_{inner} = \max(0, R_{outer} - padding)$

- Terminal Window: $R_{outer} = 16px$, padding = 20px -> $R_{inner} = \max(0, 16 - 20) = 0px$
- Scorecard Card: $R_{outer} = 14px$, padding = 16px -> $R_{inner} = 0px$
- Telemetry Chips: $R_{outer} = 8px$, padding = 6px -> $R_{inner} = 2px$
- Pill Badges: $R_{outer} = 9999px$ (capsule)

---

## 6. Motion Physics & Spring Kinematics
- Primary Transition Easing: `cubic-bezier(0.16, 1, 0.3, 1)` (snappy entrance with long readable hold)
- Spring Physics: `stiffness: 420, damping: 28, mass: 1.0`
- Snappy Micro-interactions: `stiffness: 550, damping: 32, mass: 0.8`
- Transition Rule: Transition real discrete properties only (transform, opacity, border-color). Strictly ban `transition: all`.
- Reduced Motion Contract: All keyframes and spring animations respect `@media (prefers-reduced-motion: reduce)`.
- Accessibility Contract: All interactive elements maintain clear `:focus-visible` styling (`outline: 2px solid oklch(0.75 0.16 230)` with `outline-offset: 2px`).

---

## 7. Performance Budget & Quality Gates
- Page Weight / Video Target: under 2.5 MB rendered MP4
- Render Latency: 540 frames at 30 fps (18.0 seconds) rendered in under 60 seconds
- Performance Budget: LCP < 800ms, CLS = 0, INP < 50ms
- Contrast Gate: 100% WCAG 2.1 AA compliance (ratio > 4.5:1 for text, > 3:1 for large metrics)
- Token Drift Guard: Token diff check against this DESIGN.md specification to prevent visual drift.

---

## 8. Anti-Slop Enforcement
- Anti-Slop Banned Patterns:
  - No centered hero plus 3 equal feature cards cliché
  - No generic purple or blue gradients on white
  - No buzzwords: "revolutionary", "seamless", "cutting-edge", "game-changing"
  - No pure #000 or pure #fff without anchor hue tint
  - No fake dummy diagrams or SVG doodles
- Real Artifacts Only:
  - Live benchmark numbers: 840ms crawl time, 50 rules (R01-R50), <30MB RAM
  - Exact command line: `jev-seo crawl https://akashpriyadarshi.vercel.app`
  - Real bot user-agents: `GPTBot`, `ClaudeBot`, `PerplexityBot`, `Google-Extended`
  - Exact schema: Schema.org 2026 JSON-LD validator
