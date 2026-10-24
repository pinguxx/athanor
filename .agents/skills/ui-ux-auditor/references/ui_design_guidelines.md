# Rubedo Atelier UI Design System & Component Guidelines

A concrete specification for visual styling, layout proportions, interaction tokens, and accessibility standards across the **Rubedo Atelier** dev studio and visualizer.

---

## 1. Palette & Surface Tokens

### Dark Canvas & Surface Elevations
- **Base Canvas**: `bg-slate-950` (`#020617`) — Default page and graph backdrop.
- **Card / Primary Container**: `bg-slate-900` (`#0f172a`) — Elevated surfaces for Vessel nodes, modals, telemetry logs.
- **Inner Elevated Sections**: `bg-slate-900/90` with `backdrop-blur-md` — Used for pinned topbars, inspectors, and slide-over drawers.
- **Card Borders**: Unified `border border-slate-800` (`#1e293b`), transitioning to `hover:border-slate-700` (`#334155`).

### Alchemical Status Roles
| Status Color | Visual Accent | Context of Use |
|:---|:---|:---|
| `amber-500` / `amber-400` | 🏆 Gold Transmuted | Completed step (`VesselStatus::Transmuted`), successful run completion |
| `cyan-500` / `cyan-400` | ⚡ Active Process | Currently executing step (`VesselStatus::Active`), live SSE telemetry stream |
| `purple-500` / `purple-400` | 🔮 Suspended Token | Workflow paused awaiting human/webhook resume token (`VesselStatus::Suspended`) |
| `rose-500` / `rose-400` | 🚨 Corrupted / Failure | Step execution failed or panic trapped (`VesselStatus::Corrupted`) |
| `slate-500` / `slate-400` | ⏳ Pending | Scheduled step awaiting prior node completion (`VesselStatus::Pending`) |
| `indigo-500` / `indigo-400` | ⚗️ Primary Action | Resume trigger, test reagent execution, graph refresh buttons |

---

## 2. Typography Hierarchy

| Level | Tailwind Classes | Usage Context |
|:---|:---|:---|
| **App Title** | `text-lg font-black text-white leading-tight` | Header logo brand ("Rubedo Atelier") |
| **Section Header** | `text-base font-bold text-slate-100 leading-snug` | Drawer title, Reagent test bench title |
| **Node Title** | `text-sm font-semibold text-slate-100 leading-tight` | Vessel name / step identifier |
| **Body Text** | `text-xs text-slate-300 leading-relaxed` | Descriptions, error explanations |
| **Code / Payloads** | `font-mono text-xs text-slate-200 bg-slate-950 p-2 rounded` | JSON inputs, outputs, schema preview |
| **Micro Labels** | `text-[10px] font-semibold uppercase tracking-wider text-slate-400` | Token badges, latency pills, status chips |

---

## 3. Spacing & Modal Standards

- **Graph Canvas**: Full-height viewport minus 56px header (`h-[calc(100vh-56px)]`).
- **Telemetry Drawer**: Fixed width on desktop (`w-96` or `w-[420px]`), sliding in from right with `backdrop-blur-sm`.
- **Checkpoint Modal**:
  - Max width: `max-w-2xl`
  - Max height: `max-h-[90vh]`
  - Scrollable payload editor with syntax highlighting and diff comparison.
- **Button Targets**:
  - Minimum clickable dimension: `36x36px`
  - Icon buttons have explicit `aria-label` attribute (e.g. `aria-label="Replay Vessel Execution"`).
