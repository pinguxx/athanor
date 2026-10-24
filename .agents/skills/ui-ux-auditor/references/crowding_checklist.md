# Anti-Crowding & Usability Heuristics Checklist

Use this 10-point checklist to inspect components, views, and modals to eliminate visual clutter, cognitive overload, and cramped layout patterns.

---

## The 10 Anti-Crowding Heuristics

### 1. Vertical Breathing Room Between Feed Cards
- [ ] Feed cards must have at least `16px` (`space-y-4`) to `20px` (`space-y-5`) vertical separation.
- [ ] Avoid zero-margin borders where cards bleed into each other without distinct edge boundaries.

### 2. Header & Banner Badge Consolidation
- [ ] Card top bars must not wrap into more than 2 lines on mobile screens ($\le 375\text{px}$).
- [ ] When space is constrained, truncate secondary topic labels or hide descriptive sub-labels (e.g. `hidden sm:inline`).
- [ ] Avoid displaying more than 3 high-contrast colorful pills in the same header row. Group secondary metadata into a subtle neutral text format (`text-slate-400 text-xs`).

### 3. Progressive Disclosure Over Monolithic Display
- [ ] Primary cards should showcase the core claim and the top consensus counterpoint.
- [ ] Secondary items (e.g., candidate debate list, full comment stream, Socratic fallacy analysis) MUST be tucked into an expandable drawer (`showDrawer`, `showComments`) or modal.
- [ ] Avoid expanding multiple complex drawers automatically on page load. Let the user request depth when interested.

### 4. Horizontal Scroll Safety on Mobile
- [ ] Filter bars (Topic tags, format selectors, leaderboard tables) must declare `overflow-x-auto scrollbar-none flex-nowrap`.
- [ ] Ensure `flex-shrink-0` or `shrink-0` is present on individual tab buttons so they never squeeze or truncate their text awkwardly.
- [ ] The global `main` container must never cause a horizontal window scrollbar (`overflow-x-hidden` or constrained max-width).

### 5. Line Length & Text Typography Density
- [ ] Long paragraphs should have comfortable line-height (`leading-relaxed` or `leading-normal`).
- [ ] Avoid ultra-long single-line sentences without visual breaking; split dense arguments into bullet points or distinct perspective cards.
- [ ] Ensure font size contrast is noticeable: primary text (`text-sm` or `text-base`), supporting text (`text-xs`), micro badges (`text-[10px]`).

### 6. Interactive Target Sizing (Fat-Finger Friendly)
- [ ] All clickable buttons, tabs, and avatar icons must have a minimum interactive touch target of $36 \times 36\text{px}$ on mobile, preferably $44 \times 44\text{px}$ with padding.
- [ ] Avoid placing two dense, small icon buttons directly touching each other without gap spacing (`gap-1.5` or `gap-2`).

### 7. Modal Information Density
- [ ] Modals must never exceed 90% of screen height (`max-h-[90vh]`).
- [ ] Modals must have distinct, un-crowded sections:
  1. Fixed Header (title + close button).
  2. Scrollable Body with clean whitespace (`space-y-4 sm:space-y-5`).
  3. Optional Fixed Action Footer.
- [ ] Multi-faceted modals (e.g., `UserProfileModal`, `AssemblyDetailModal`) must use segmented tabs rather than rendering posts, counterpoints, badges, stats, and diet in one continuous, overwhelming wall.

### 8. Visual Hierarchy in Complex Data Grids
- [ ] In stats grids (e.g. Profile stats, leaderboards), use 2 columns on mobile and 4 columns on desktop (`grid grid-cols-2 sm:grid-cols-4 gap-3`).
- [ ] Emphasize the numerical metric with bold prominent text (`text-lg font-black text-indigo-400`), keeping the label short and subtle (`text-[11px] text-slate-400`).

### 9. Contrast & Cognitive Load
- [ ] Background surfaces must use harmonious dark tones (`slate-950` vs `slate-900` vs `slate-800`), avoiding jarring neon backgrounds or uncoordinated borders.
- [ ] Action buttons must have one clear primary call-to-action per screen (e.g., indigo for primary submit), while secondary actions remain neutral (`bg-slate-800 hover:bg-slate-700`).

### 10. Empty State Reassurance
- [ ] When a list or filter returns zero results (search query, Following tab with 0 follows, candidate arena with 0 rebuttals), provide an inviting, spacious empty state.
- [ ] An ideal empty state contains:
  - An uncluttered icon.
  - A friendly title.
  - A concise explanation of why it is empty.
  - A clear, one-click action button to reset filters or explore content.
