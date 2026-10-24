---
name: ui-ux-auditor
description: >-
  Audits user interface (UI) and user experience (UX) for the Rubedo Atelier Dev Studio.
  Use this skill whenever asked to review UI consistency, audit usability, check for visual crowding,
  evaluate DAG workflow layout breathing room, assess dark theme design system fidelity,
  inspect responsiveness, or improve accessibility and interaction affordances.
---

# UI/UX Usability & Consistency Auditor (Rubedo Atelier)

An expert frontend design, usability, and visual hygiene skill designed to ensure the **Rubedo Atelier** developer studio and visualizer is clean, intuitive, well-spaced, and visually harmonious across desktop and mobile viewports.

---

## 🎨 Core Design Principles for Rubedo Atelier

### 1. Visual Hierarchy & Spacing (DAG Workflows & Telemetry)
- **Breathing Room**: Avoid cramped nodes on the workflow canvas. Vessel nodes must maintain at least `24px` horizontal and vertical clearance.
- **Progressive Disclosure**:
  - The primary canvas showcases the DAG topology and node execution states (`Pending`, `Active`, `Suspended`, `Transmuted`, `Corrupted`).
  - Deep telemetry details (JSON input/output payloads, stack traces, instruction fuel consumption, token usage) belong in expandable drawers, inspectable tooltips, or dedicated checkpoint modals.
  - Never dump every raw JSON payload onto the canvas at once.
- **Scannable Information Architecture**:
  - Active nodes and current execution pointers must be immediately distinct (e.g. glowing cyan border or pulse indicator).
  - Terminal states must be unmistakable: `Transmuted` (gold/amber `#f59e0b`), `Suspended` (purple `#a855f7`), `Corrupted` (rose `#f43f5e`).

### 2. Design System Consistency (Alchemical Slate Tokens)
- **Unified Palette**:
  - Canvas / Background: `bg-slate-950` (`#020617`).
  - Container / Card: `bg-slate-900` (`#0f172a`).
  - Elevated Drawers & Pinned Toolbars: `bg-slate-900/90` with `backdrop-blur-md`.
  - Borders: `border-slate-800` (`#1e293b`), hover: `border-slate-700`.
  - Neutral Typography: `text-white` (titles), `text-slate-200` (body), `text-slate-400` / `text-slate-500` (subtext, timestamps, inactive states).
- **Alchemical Semantic Accents**:
  - 🏆 **Gold / Amber** (`amber-400` / `amber-500`): Successful transmutation (`VesselStatus::Transmuted`).
  - ⚡ **Cyan** (`cyan-400` / `cyan-500`): Currently running step (`VesselStatus::Active`), real-time SSE streaming.
  - 🔮 **Purple** (`purple-400` / `purple-500`): Suspended workflow awaiting external resume token (`VesselStatus::Suspended`).
  - 🚨 **Rose** (`rose-500` / `rose-600`): Step failure or panic caught (`VesselStatus::Corrupted`).
  - 🌿 **Emerald** (`emerald-400` / `emerald-500`): Completed Opus runs, healthy system status.

### 3. Interaction Affordances & State Feedback
- **Clickable Affordances**:
  - All interactive elements (nodes, replay buttons, resume trigger, tabs) MUST have `cursor-pointer`.
  - Subtle hover feedback: background elevation shift (`hover:bg-slate-800`) or border illumination (`hover:border-slate-700`).
- **Accessible Controls**:
  - Icon-only buttons (close, replay, zoom, copy token) MUST include descriptive `aria-label` tags.
  - Keyboard navigation: `Escape` closes drawers/modals, `Space`/`Enter` activates buttons.

---

## 🛠️ Automated UI Audit Tool

Run the automated UI auditor script:
```powershell
powershell -ExecutionPolicy Bypass -File .agents/skills/ui-ux-auditor/scripts/audit_ui.ps1
```

For exhaustive audit guidelines, consult:
- [Atelier Anti-Crowding Checklist](./references/crowding_checklist.md)
- [Atelier Design System & Token Guidelines](./references/ui_design_guidelines.md)
