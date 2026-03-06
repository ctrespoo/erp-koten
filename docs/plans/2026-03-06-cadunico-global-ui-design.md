# CadUnico Global UI Design

Date: 2026-03-06
Status: Approved
Scope: Global UI standardization with immediate focus on CadUnico (list, create form, delete confirmation)

## Context
User-reported issues reproduced in browser via MCP:
- Row focus animation in list table feels unstable and visually noisy.
- Keyboard navigation breaks inside row action dropdown (table navigation steals focus).
- Delete confirmation visual quality is poor and inconsistent.
- CadUnico create form CSS is not applied.
- Overall product needs better UX/UI consistency under a minimalist, functional direction.

Technical root cause identified:
- A CSS syntax break in `assets/styles/app.css` (missing closing brace in shared button block) interrupts subsequent rule parsing and degrades multiple components, including form and modal styling.

## Design Direction
Minimalist operational UI:
- Low decoration, high legibility, predictable keyboard behavior.
- Dark palette preserved, but with cleaner surfaces and reduced ornamental gradients.
- Consistent interaction states (`hover`, `focus-visible`, `active`, `disabled`, `invalid`).
- Functional hierarchy first: data readability, action clarity, stable focus transitions.

## Approaches Considered
1. System-first:
- Build a complete design system first, then migrate screens.
- Pros: strongest consistency.
- Cons: slower initial delivery.

2. Screen-first:
- Fix each screen directly without shared primitives.
- Pros: quick local output.
- Cons: high risk of style drift and future regressions.

3. Hybrid (chosen):
- Create a minimal shared foundation (tokens + critical primitives) and apply immediately to CadUnico, then expand globally.
- Rationale: best balance between speed and sustainable consistency.

## UX/UI Spec

### 1. Visual Architecture
- Consolidate design tokens in global stylesheet:
  - Color roles: background layers, text tiers, accent, destructive, border, focus.
  - Spacing scale and radii.
  - Motion timings and shadow levels.
- Define lightweight primitives:
  - Buttons (primary, secondary, destructive)
  - Inputs and labels
  - Tabs
  - Table states
  - Dialog/modal shell

### 2. List/Table Behavior
- Replace row highlight animation that changes layout metrics.
- Use stable active-row treatment:
  - Subtle background lift
  - Non-displacing focus ring or inset outline
- Preserve readable row density and action-column alignment.

### 3. Row Action Dropdown (Keyboard)
- Introduce explicit menu-open state.
- While open:
  - ArrowUp/ArrowDown navigate menu items only.
  - Enter activates focused item.
  - Escape closes menu and restores focus to source row.
- Outside click closes menu without focus confusion.
- Reposition menu with viewport-aware bounds (flip/shift when needed).

### 4. Delete Confirmation Dialog
- Minimal dialog with clear structure:
  - Context eyebrow
  - Strong title
  - Specific confirmation text (entity name)
  - Secondary cancel + destructive confirm actions
- Improve overlay depth and spacing consistency.
- Keyboard behavior:
  - Focus enters dialog on open
  - Escape closes
  - Predictable focus restoration on close

### 5. CadUnico Form
- Fix CSS parsing issue to restore style application.
- Revalidate classes and responsive behavior:
  - `cadunico-form`, `tab-strip`, `tab-button`, `tab-panel`, `field`, `form-footer`
- Ensure tab active/focus states are clear and consistent with global primitives.
- Keep form grid readable on both desktop and mobile.

### 6. Global Standardization Pass
After CadUnico baseline is corrected, apply same primitives/tokens to remaining screens:
- Unified typography scale
- Unified spacing rhythm
- Unified state colors and focus handling
- Unified action hierarchy (primary/secondary/destructive)

## Testing and Verification
- Automated:
  - JS tests for list keyboard flows and open-menu key handling.
  - Regression tests for create form styled rendering path.
- Manual via MCP:
  - Desktop and mobile viewport checks.
  - Row focus stability.
  - Dropdown in-viewport placement.
  - Dialog visual and keyboard behavior.
  - Form tabbing and shortcut behavior.

## Rollout Plan
1. Implement token/primitives baseline + CadUnico fixes first.
2. Validate behavior and visuals in CadUnico end-to-end.
3. Apply same baseline across remaining screens.
4. Run final UX consistency pass and regressions.

## Approval Log
- Visual direction: Minimalist functional UI (approved by user).
- Scope: Global standardization (approved by user).
- Approach: Hybrid (approved by user).
- Design sections 1, 2, and 3: approved by user.
