# Anti-Crowding & Usability Heuristics for Rubedo Atelier

Use this 10-point checklist to inspect components, views, and drawers in the **Rubedo Atelier** visualizer to eliminate visual clutter and cognitive overload.

---

## The 10 Anti-Crowding Heuristics

### 1. Canvas Node Separation
- [ ] Workflow DAG nodes must have at least `24px` horizontal and vertical spacing to prevent overlapping edge lines.
- [ ] Node cards must declare fixed or clamped dimensions (`w-64 max-w-xs`) so long step titles don't stretch the graph unevenly.

### 2. Header & Metrics Bar Consolidation
- [ ] Top telemetry toolbar must not wrap awkwardly on smaller viewports.
- [ ] Group cumulative metrics (total steps, elapsed time, total tokens consumed) into clean pill badges (`bg-slate-800 text-xs px-3 py-1 rounded-full`).
- [ ] Never display more than 3 competing high-contrast colored badges in the header row.

### 3. Progressive Disclosure of Payloads
- [ ] Primary canvas nodes display: Vessel ID, Status Badge, and Duration.
- [ ] Raw input and output JSON payloads MUST be tucked into an expandable drawer or inspect modal.
- [ ] Long JSON trees should default to collapsed top-level keys with a one-click "Expand All" / "Copy JSON" control.

### 4. Live Telemetry Stream Scannability
- [ ] Real-time SSE execution logs must use comfortable line-height (`leading-relaxed`) and monospace typography (`font-mono text-xs`).
- [ ] Auto-scroll on new events must include a user pause toggle ("Pause stream on hover or manual scroll") so developers can read historical traces without getting yanked to the bottom.

### 5. Checkpoint Replay Modal Density
- [ ] The time-travel checkpoint inspector modal must not exceed `90vh` height (`max-h-[90vh]`).
- [ ] Provide side-by-side or tabbed "Before / After" diff views rather than unformatted JSON dumps.
- [ ] The "Resume Step with Token" form must visually isolate the token input, payload editor, and submit button.

### 6. Interactive Target Sizing
- [ ] All clickable nodes, drawer close toggles, and copy buttons must have a minimum interactive touch target of $36 \times 36\text{px}$.
- [ ] Icon buttons must declare explicit `aria-label` tags for screen readers.

### 7. Interactive Reagent Test Bench
- [ ] Dynamic JSON Schema form fields must provide clear inline validation errors when inputs violate schema types.
- [ ] Form submit buttons must display an active loading spinner while the reagent is transmuting.

### 8. Visual Hierarchy & Status Contrast
- [ ] Background surfaces must use harmonious dark tones (`bg-slate-950` canvas, `bg-slate-900` card containers).
- [ ] Node status must be color-coded according to the alchemical palette:
  - Gold / Amber: `Transmuted`
  - Cyan: `Active` (animated pulse)
  - Purple: `Suspended`
  - Rose: `Corrupted`
  - Slate: `Pending`

### 9. Empty & Uninitialized States
- [ ] When an Opus run has not yet started or has no recorded events, display an inviting empty canvas state with:
  - An uncluttered alchemical flask or DAG icon.
  - A helpful title ("No active workflow run").
  - Clear instructions on triggering a run via CLI or API.

### 10. Horizontal Scroll Safety
- [ ] Canvas panning must be constrained or explicit. The outer application shell must never cause an accidental horizontal window scrollbar.
