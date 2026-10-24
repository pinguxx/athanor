---
name: ui-ux-auditor
description: >-
  Audits user interface (UI) and user experience (UX) across web applications.
  Use this skill whenever asked to review UI consistency, audit usability, check for visual crowding,
  evaluate layout breathing room, assess design system fidelity (Tailwind tokens, colors, typography),
  inspect mobile responsiveness, or improve accessibility and interaction affordances.
---

# UI/UX Usability & Consistency Auditor

An expert frontend design, usability, and visual hygiene skill designed to ensure user interfaces look clean, consistent, well-spaced, intuitive, and visually harmonious across all viewports.

---

## 🎨 Core Design Principles & Heuristics

### 1. Visual Hierarchy & Spacing (Anti-Crowding & Breathing Room)
- **Generous Whitespace**: Avoid walls of dense text or cramped clusters of controls. Use consistent padding scales (`p-3 sm:p-4`, `p-5 sm:p-6`) and gaps (`gap-2 sm:gap-3`, `gap-4`).
- **Progressive Disclosure**:
  - Keep primary views (cards, stream items) clean and digestible at a glance.
  - Secondary or deep details (e.g., candidate debate arenas, algorithmic synthesis, comment lists, report options) belong in expandable drawers, collapsible sections, or dedicated modals.
  - Never dump every action, filter, and nested sub-menu into the initial view at once.
- **Scannable Information Architecture**:
  - Primary headline/take must dominate visually (`text-sm font-bold` or `text-base font-bold`).
  - Metadata (author avatar, handle, time elapsed, topic tag) should sit on a dedicated, compact row (`text-xs text-slate-400`).
  - Key scores and badges (bridge scores, Deliberative Elo, consensus crown) should use crisp, distinct accent pills rather than plain unstyled text.

### 2. Design System Consistency (TwoSides Tokens)
- **Palette Consistency**:
  - Backgrounds: Dark mode default with `bg-slate-950` (page canvas), `bg-slate-900` (cards and containers), `bg-slate-950/80` or `bg-slate-900/90` (drawers & inner headers).
  - Borders: Unified border tone `border-slate-800` (hover: `border-slate-700` or accent highlight).
  - Neutral Typography: `text-white` (key titles), `text-slate-100` (card text), `text-slate-300` (body), `text-slate-400` / `text-slate-500` (subtext, timestamps, inactive states). Avoid mixing arbitrary gray scales (do not mix `text-gray-` or `text-zinc-` when `slate` is the design system base).
  - Semantic Accent Roles:
    - ⚡ **Indigo**: Primary brand actions, votes, debate navigation, campus verify.
    - 🏆 **Amber**: Steelman achievements, consensus badges, bridge scores, warning highlights.
    - 🌿 **Emerald**: Casual humor format, agreement consensus, online status, positive toasts.
    - 💬 **Cyan**: Following feed tab, user profiles, followers count.
    - 🔮 **Purple**: Perspective Diet, Socratic pre-flight checks, Community Assemblies.
    - 🚨 **Rose**: Quarantined items, safety reports, dispute/disagree stances, error states.
- **Border Radii & Shape Grammar**:
  - Avatars & Status Indicators: Fully circular (`rounded-full`).
  - Cards & Modals: Generously rounded (`rounded-2xl`).
  - Buttons & Input Fields: Moderately rounded (`rounded-xl` or `rounded-lg`).
  - Micro-badges & Chips: Soft-cornered pills (`rounded-md` or `rounded-full px-2 py-0.5`).

### 3. Interaction Affordances & State Feedback
- **Clickable Affordances**:
  - Every interactive element (buttons, tabs, author headers, topic pills) MUST display `cursor-pointer`.
  - Visual hover feedback: Subtle background shift (`hover:bg-slate-800`), border illumination (`hover:border-slate-700`), or text brighten (`hover:text-white`).
  - Active/Selected states: Distinct contrast (e.g., solid accent fill with crisp text and optional shadow).
- **Loading & Empty State Empathy**:
  - Empty states must never look like an application crash. Provide a dedicated icon (`lucide-react`), an explanatory title, a reassuring subtext explaining *why* it is empty, and a clear call-to-action button (e.g., "Explore All Takes", "Create First Post").
  - Loading states should display a centered spinner or skeleton animation.
- **Action Confirmation & Error Recovery**:
  - Immediate feedback on vote cast, follow toggle, or comment submission.
  - Floating toast notifications for transient confirmations (`toast.type === 'success' | 'warn' | 'info'`).

### 4. Responsive & Mobile Adaptability
- **Touch-Friendly Targets**: Minimum interactive area $\ge 40 \times 40\text{px}$ (or padded appropriately `p-2 sm:p-2.5`).
- **Horizontal Scroll Containers**: Navigation tabs, filter chips, and leaderboard tables must scroll horizontally with `overflow-x-auto scrollbar-none flex-nowrap` on mobile without causing body-level horizontal page overflow.
- **Stacking Layouts**: Split-screen components (e.g. `SplitCard`) must stack vertically (`flex flex-col md:grid md:grid-cols-2`) on narrow screens to prevent cramped text columns.

### 5. Accessibility (a11y) & Keyboard Navigation
- **Screen Reader Clarity**: Icon-only buttons must provide `aria-label` or `title` describing the action (e.g. `aria-label="Close modal"`).
- **Focus Rings**: Interactive inputs and buttons must include visible focus outlines (`focus-visible:ring-2 focus-visible:ring-indigo-400 focus-visible:outline-none`).
- **Modal Accessibility**: Modals must trap focus, handle <kbd>Escape</kbd> dismissal, close on backdrop click, and support `role="dialog" aria-modal="true"`.

---

## 🔍 The 4-Phase UI/UX Audit Protocol

### Phase 1: Visual Hygiene & Token Inspection
1. Scan component files for inconsistent utility classes:
   - Run the automated UI audit script:
     ```powershell
     ./scripts/audit_ui.ps1
     ```
2. Verify color harmony: Ensure no accidental hardcoded inline styles (`style={{ ... }}`) bypassing Tailwind.
3. Check typography: Ensure heading tags (`<h1>` through `<h5>`) follow a logical, non-skipping hierarchy.

### Phase 2: Anti-Crowding & Information Density Evaluation
1. Measure vertical and horizontal rhythm: Are cards suffocated with too many small pills and icons side-by-side?
2. Check card boundaries: Is there sufficient breathing room between cards (`space-y-4` or `space-y-5`)?
3. Review progressive disclosure: Are heavy sub-features (e.g. counterpoint debate drawers, fallacy reports) collapsed by default?

### Phase 3: Usability & Interactive Flow Review
1. Walk through the critical user journeys:
   - Feed browsing & filter switching (`All` $\to$ `Perspectives` $\to$ `Casual` $\to$ `Assemblies` $\to$ `AI Arena` $\to$ `Following`).
   - Voting & dual-axis rating interaction.
   - Profile exploration & following toggle.
   - Post creation and counterpoint submission.
2. Verify feedback states: Do buttons disable (`disabled:opacity-50`) during submission to prevent duplicate clicks?

### Phase 4: Synthesis & Actionable Remediation
1. Benchmark against [UI Design Guidelines](./references/ui_design_guidelines.md) and [Crowding Checklist](./references/crowding_checklist.md).
2. Produce a clear, structured UI audit report with side-by-side or before/after component diffs.

---

## 📋 UI/UX Audit Report Template

```markdown
# 🎨 UI/UX Design & Usability Audit Report

## 🌟 Executive Summary
- **Visual Hygiene & Consistency**: [ EXCELLENT | REFINEMENT RECOMMENDED | INCONSISTENT ]
- **Layout Breathing Room & Anti-Crowding**: [ CLEAN & SPACIOUS | ACCEPTABLE | TOO CROWDED ]
- **Key Takeaway**: [2-3 sentence overview of the aesthetic and functional posture]

---

## 🔎 Detailed Findings Matrix

| Component | Area | Issue Description | Severity | Recommendation |
|:---|:---|:---|:---|:---|
| `SplitCard.tsx` | Visual Density | Header badge cluster wraps into 3 lines on mobile | Medium | Use compact chip layout and hide secondary label on small screens |
| `Header.tsx` | Consistency | User menu item was missing icon alignment | Low | Standardize padding with `px-2.5 py-1.5` |
| `UserProfileModal.tsx` | Usability | Stats cards had cramped margins on narrow viewports | Medium | Switch from 4-col to 2x2 grid with `grid-cols-2 sm:grid-cols-4` |

---

## 🛠️ Concrete Visual & Code Remediations

### 1. [Component Name] — Remediation Diff
**Problem**: [Explain why the UI was crowded or inconsistent]
```diff
- <div className="p-1 flex space-x-1">
+ <div className="p-3 sm:p-4 flex flex-wrap items-center gap-2">
```
**Result**: [Explain the improved aesthetic and user experience]
```
