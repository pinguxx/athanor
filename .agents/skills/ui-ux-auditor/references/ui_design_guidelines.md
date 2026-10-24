# TwoSides UI Design System & Component Guidelines

A concrete specification for visual styling, layout proportions, interaction tokens, and accessibility standards across the TwoSides web client.

---

## 1. Palette & Surface Tokens

### Dark Canvas & Surface Elevations
- **Base Canvas**: `bg-slate-950` (`#020617`) — Default page backdrop.
- **Card / Primary Container**: `bg-slate-900` (`#0f172a`) — Main elevated surfaces for cards, modals, dropdowns.
- **Inner Elevated Sections**: `bg-slate-950/80` or `bg-slate-900/90` with `backdrop-blur-md` — Used for pinned headers, card top banners, sub-drawers, and footers.
- **Card Borders**: Unified `border border-slate-800` (`#1e293b`), transitioning to `hover:border-slate-700` (`#334155`).

### Semantic Accent Roles
| Color Token | Visual Accent | Context of Use |
|:---|:---|:---|
| `indigo-600` / `indigo-400` | ⚡ Primary Action | Main navigation, active tabs, dual-axis quality sliders, primary submission buttons |
| `amber-500` / `amber-400` | 🏆 Consensus & Energy | Vote energy balance, Deliberative Elo leaderboard, consensus crowns, Master Steelman badges |
| `emerald-600` / `emerald-400`| 🌿 Casual & Agreement | Casual banter format, positive consensus indicators, online connectivity status, success toasts |
| `cyan-600` / `cyan-400` | 💬 Following & Social | Following tab filter, user profile modal links, follow/following toggles, follower counts |
| `purple-600` / `purple-400` | 🔮 Epistemic Intelligence| Community Assemblies, Perspective Diet, Socratic pre-flight checks, AI Dialectic synthesis |
| `rose-600` / `rose-400` | 🚨 Safety & Opposition | Disagreement stances, safety violation report flags, quarantined item tombstones |

---

## 2. Typography Hierarchy

| Level | Tailwind Classes | Usage Context |
|:---|:---|:---|
| **Page Title** | `text-lg sm:text-xl font-black text-white leading-tight` | Header brand logo, major modal titles |
| **Section Header** | `text-base font-bold text-slate-100 leading-snug` | Assembly detail headings, leaderboard title |
| **Card Thesis** | `text-sm sm:text-base font-bold text-slate-100 leading-relaxed` | Primary take arguments, counterpoint claims |
| **Body Text** | `text-xs sm:text-sm text-slate-200 leading-relaxed` | Counter-arguments, long explanations, synthesis paragraphs |
| **Subtext / Meta** | `text-xs text-slate-400 leading-tight` | Author handles, timestamps, category descriptions |
| **Micro Labels** | `text-[10px] sm:text-[11px] font-semibold uppercase tracking-wider` | Chip pills, tag badges, voting category descriptors |

---

## 3. Spacing & Padding Scale

- **App Canvas Padding**:
  - Desktop: `max-w-4xl mx-auto px-4 py-5`
  - Mobile: `px-3 py-3`
- **Card Padding**:
  - Card Exterior: `space-y-4` or `space-y-5` separation between cards in feeds.
  - Card Interior: `p-3.5 sm:p-5`
  - Card Banner / Header: `px-3 py-2 sm:px-4 sm:py-2.5 border-b border-slate-800`
- **Modal Containers**:
  - Maximum width: `max-w-xl` for compact alerts/create forms, `max-w-2xl` for profiles/assemblies.
  - Height limit: `max-h-[90vh] flex flex-col overflow-hidden`
  - Modal body: `p-4 sm:p-6 overflow-y-auto space-y-4`

---

## 4. Button & Interactive Affordances

### Standard Button Variants
1. **Primary Solid**:
   ```html
   <button className="px-3.5 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-bold text-xs shadow-sm transition cursor-pointer flex items-center space-x-1.5 focus-visible:ring-2 focus-visible:ring-indigo-400">
   ```
2. **Secondary / Subtle**:
   ```html
   <button className="px-3 py-1.5 rounded-xl bg-slate-900 hover:bg-slate-800 border border-slate-800 text-slate-300 hover:text-white font-medium text-xs transition cursor-pointer flex items-center space-x-1.5">
   ```
3. **Pill / Filter Tab**:
   ```html
   <button className="px-2.5 py-1 sm:px-3 sm:py-1.5 rounded-lg text-xs font-semibold transition cursor-pointer shrink-0 whitespace-nowrap">
   ```
4. **Icon-Only Action**:
   ```html
   <button aria-label="Action description" className="p-1.5 rounded-lg text-slate-400 hover:text-slate-200 hover:bg-slate-800 transition cursor-pointer">
   ```

---

## 5. Modal & Drawer Anatomy Guidelines

1. **Header**:
   - Distinct icon + title + short subtitle.
   - Fixed close button top-right (<kbd>X</kbd>) with `aria-label="Close dialog"`.
2. **Scroll Area**:
   - Only the modal body should scroll (`overflow-y-auto`); the header and action footer remain pinned.
3. **Backdrop & Dismissal**:
   - `fixed inset-0 z-50 bg-slate-950/70 backdrop-blur-sm flex items-center justify-center p-3 sm:p-4`
   - Closes on backdrop click or <kbd>Escape</kbd>.
