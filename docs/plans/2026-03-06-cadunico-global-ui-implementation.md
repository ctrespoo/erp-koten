# CadUnico Global UI Standardization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix CadUnico UX/UI regressions (row focus animation, dropdown keyboard behavior, delete confirmation visual quality, missing form CSS) and establish a reusable minimalist UI baseline for global standardization.

**Architecture:** Implement a hybrid approach: first create a small shared visual foundation (tokens + critical primitives in `app.css`), then apply behavior and styling fixes to CadUnico list/form/dialog flows, and finally extend the same primitives across remaining screens without changing business logic. Keyboard-first interaction is treated as a first-class behavior and validated with targeted unit tests.

**Tech Stack:** Rust (Axum + Askama), HTMX, vanilla JavaScript modules, CSS, Vitest (jsdom), Cargo tests.

---

Execution skills to use during implementation: `@test-driven-development`, `@systematic-debugging`, `@verification-before-completion`, `@requesting-code-review`.

### Task 1: Lock Dropdown Keyboard Behavior with Failing Tests

**Files:**
- Modify: `assets/scripts/cadunico-list.test.js`
- Test: `assets/scripts/cadunico-list.test.js`

**Step 1: Write the failing test**

```js
it("menu navigation should consume ArrowUp/ArrowDown when row menu is open", () => {
  mountList();
  const rows = Array.from(document.querySelectorAll("[data-row]"));

  document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
  document.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));

  const menuButton = document.querySelector("[data-row-delete]");
  expect(document.activeElement).toBe(menuButton);

  document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));

  // Active row must stay the same while menu is open
  expect(document.activeElement).toBe(menuButton);
  expect(rows[0].dataset.rowActive).toBe("true");
});
```

**Step 2: Run test to verify it fails**

Run: `bun x vitest run assets/scripts/cadunico-list.test.js`
Expected: FAIL in new menu-keyboard test because current global Arrow handler still navigates table rows.

**Step 3: Write minimal implementation**

```js
// inside onKeyDown
const menu = currentPopover(root);
if (menu) {
  const menuButtons = Array.from(menu.querySelectorAll("button:not([disabled])"));
  // handle ArrowUp/ArrowDown/Enter/Escape here and return early
}
```

**Step 4: Run test to verify it passes**

Run: `bun x vitest run assets/scripts/cadunico-list.test.js`
Expected: PASS for new test and existing list tests.

**Step 5: Commit**

```bash
git add assets/scripts/cadunico-list.test.js assets/scripts/cadunico-list.js
git commit -m "test: cover keyboard navigation while row menu is open"
```

### Task 2: Fix Row Menu Focus Lifecycle and Viewport Positioning

**Files:**
- Modify: `assets/scripts/cadunico-list.js`
- Modify: `assets/scripts/cadunico-list.test.js`
- Test: `assets/scripts/cadunico-list.test.js`

**Step 1: Write the failing tests**

```js
it("Escape should close menu and restore focus to source row", () => {
  mountList();
  const row = document.querySelector("[data-row]");

  document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
  document.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
  document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));

  expect(document.activeElement).toBe(row);
  expect(document.querySelector("[data-row-menu-popover]")).toBeNull();
});
```

```js
it("popover should clamp position inside viewport bounds", () => {
  // direct unit for positionPopover helper after exporting deterministic variant
  const result = clampPopoverPosition({ top: 900, left: 1400 }, { width: 220, height: 120 }, { width: 1280, height: 720 });
  expect(result.left).toBeLessThanOrEqual(1280 - 220 - 16);
  expect(result.top).toBeLessThanOrEqual(720 - 120 - 16);
});
```

**Step 2: Run test to verify it fails**

Run: `bun x vitest run assets/scripts/cadunico-list.test.js`
Expected: FAIL for position clamp helper and/or lifecycle assertion.

**Step 3: Write minimal implementation**

```js
function clamp(value, min, max) {
  return Math.min(Math.max(value, min), max);
}

function clampPopoverPosition(anchor, menuRect, viewport) {
  return {
    top: clamp(anchor.top, 16, viewport.height - menuRect.height - 16),
    left: clamp(anchor.left, 16, viewport.width - menuRect.width - 16),
  };
}
```

**Step 4: Run test to verify it passes**

Run: `bun x vitest run assets/scripts/cadunico-list.test.js`
Expected: PASS including new focus/position tests.

**Step 5: Commit**

```bash
git add assets/scripts/cadunico-list.js assets/scripts/cadunico-list.test.js
git commit -m "fix: stabilize row menu focus flow and viewport positioning"
```

### Task 3: Add Regression Tests for CadUnico Form and Dialog Markup Shell

**Files:**
- Modify: `tests/cadunico_routes.rs`
- Test: `tests/cadunico_routes.rs`

**Step 1: Write the failing tests**

```rust
#[tokio::test]
async fn get_cadunico_create_should_render_form_shell_classes() {
    let app = test_app();
    let response = app.oneshot(Request::builder().uri("/cadunico/criar").body(Body::empty()).unwrap()).await.unwrap();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("class=\"cadunico-form\""));
    assert!(html.contains("class=\"tab-strip\""));
    assert!(html.contains("class=\"form-footer\""));
}
```

```rust
#[tokio::test]
async fn get_cadunico_index_should_render_delete_dialog_actions_semantics() {
    // assert destructive + cancel action hooks are present
}
```

**Step 2: Run test to verify it fails**

Run: `DATABASE_URL=postgres://postgres:postgres@localhost/erp_koten cargo test get_cadunico_create_should_render_form_shell_classes -- --exact`
Expected: FAIL until assertions match current template shell and action hooks.

**Step 3: Write minimal implementation (if needed)**

```rust
// adjust template strings only if required by assertions;
// keep behavior unchanged in this task.
```

**Step 4: Run tests to verify they pass**

Run: `DATABASE_URL=postgres://postgres:postgres@localhost/erp_koten cargo test cadunico -- --nocapture`
Expected: PASS for new route tests and existing cadunico route tests.

**Step 5: Commit**

```bash
git add tests/cadunico_routes.rs templates/cadunico/_delete_dialog.html templates/cadunico/create.html
git commit -m "test: add route regressions for form shell and delete dialog markup"
```

### Task 4: Repair CSS Parsing Break and Restore Form Styling

**Files:**
- Modify: `assets/styles/app.css`
- Test: `assets/scripts/cadunico-form.test.js`
- Modify: `assets/scripts/cadunico-form.test.js`

**Step 1: Write the failing test**

```js
it("create form should keep first tab active on bootstrap", () => {
  document.body.innerHTML = `
    <main data-cadunico-root>
      <form data-cadunico-form>
        <button data-tab-trigger data-tab-id="dados-principais"></button>
        <button data-tab-trigger data-tab-id="endereco"></button>
        <section class="tab-panel" data-tab-panel="dados-principais"></section>
        <section class="tab-panel" data-tab-panel="endereco"></section>
      </form>
      <div id="cadunico-modal-root"></div>
    </main>
  `;

  bootstrapCadUnicoForm(document.querySelector("[data-cadunico-root]"));
  expect(document.querySelector('[data-tab-trigger][data-tab-id="dados-principais"]').classList.contains("is-active")).toBe(true);
});
```

**Step 2: Run test to verify baseline behavior**

Run: `bun x vitest run assets/scripts/cadunico-form.test.js`
Expected: PASS (JS behavior guard) before CSS changes.

**Step 3: Write minimal implementation**

```css
.row-menu-popover button,
.list-pagination button,
.delete-dialog__actions button {
  border: 1px solid var(--line);
  border-radius: 999px;
  background: rgba(11, 13, 15, 0.48);
  color: var(--ink);
  cursor: pointer;
  padding: 0.75rem 1rem;
  font: inherit;
}
```

Also in same task:
- Keep form primitives (`cadunico-form`, `tab-strip`, `tab-button`, `tab-panel`, `.field`, `.form-footer`) in the parsed CSS region.
- Ensure mobile rules still include `field--wide` fallback.

**Step 4: Run tests to verify pass**

Run: `bun x vitest run assets/scripts/cadunico-form.test.js`
Expected: PASS for all form keyboard/validation tests.

**Step 5: Commit**

```bash
git add assets/styles/app.css assets/scripts/cadunico-form.test.js
git commit -m "fix: restore cadunico form styles by repairing css parsing"
```

### Task 5: Apply Minimalist Primitives to Table Rows and Delete Dialog

**Files:**
- Modify: `assets/styles/app.css`
- Modify: `templates/cadunico/_delete_dialog.html`
- Modify: `templates/cadunico/_list.html`
- Test: `assets/scripts/cadunico-list.test.js`

**Step 1: Write the failing tests**

```js
it("Enter on focused menu item should open delete dialog", () => {
  mountList();
  document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
  document.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
  document.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));

  expect(document.querySelector("[data-delete-dialog]").open).toBe(true);
});
```

```js
it("active row should remain the same when opening menu", () => {
  mountList();
  const rows = Array.from(document.querySelectorAll("[data-row]"));
  document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
  document.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
  expect(rows[0].dataset.rowActive).toBe("true");
});
```

**Step 2: Run test to verify it fails**

Run: `bun x vitest run assets/scripts/cadunico-list.test.js`
Expected: FAIL until Enter/menu behavior is wired correctly.

**Step 3: Write minimal implementation**

```css
.cadunico-list-table tbody tr[data-row-active="true"] td {
  background: color-mix(in srgb, var(--accent) 12%, transparent);
}

.cadunico-list-table tbody tr[data-row-active="true"] {
  outline: 2px solid color-mix(in srgb, var(--focus) 55%, transparent);
  outline-offset: -2px;
}

.delete-dialog__actions .button--danger {
  background: color-mix(in srgb, #b13a3a 72%, #7f2929 28%);
}
```

And update template buttons for semantic classes:

```html
<button type="submit" value="cancel" class="button button--secondary">Cancelar</button>
<button type="submit" data-confirm-delete class="button button--danger">Excluir</button>
```

**Step 4: Run tests to verify pass**

Run: `bun x vitest run assets/scripts/cadunico-list.test.js`
Expected: PASS for all list keyboard and dialog-open tests.

**Step 5: Commit**

```bash
git add assets/styles/app.css templates/cadunico/_delete_dialog.html templates/cadunico/_list.html assets/scripts/cadunico-list.test.js assets/scripts/cadunico-list.js
git commit -m "feat: apply minimalist row focus and delete dialog primitives"
```

### Task 6: Global Standardization Pass Across Shared Shell Components

**Files:**
- Modify: `assets/styles/app.css`
- Modify: `templates/layouts/app.html`
- Modify: `templates/cadunico/index.html`
- Modify: `templates/cadunico/create.html`
- Test: `tests/cadunico_routes.rs`

**Step 1: Write failing regression test**

```rust
#[tokio::test]
async fn get_cadunico_pages_should_expose_consistent_shell_classes() {
    // assert shell/page-header/shortcut-help classes appear on both /cadunico and /cadunico/criar
}
```

**Step 2: Run test to verify it fails**

Run: `DATABASE_URL=postgres://postgres:postgres@localhost/erp_koten cargo test get_cadunico_pages_should_expose_consistent_shell_classes -- --exact`
Expected: FAIL until both templates align with shared shell conventions.

**Step 3: Write minimal implementation**

```html
<main class="shell shell--wide ...">...</main>
<header class="page-header ...">...</header>
<p class="shortcut-help shortcut-help--inline">...</p>
```

```css
:root {
  --surface-0: #111315;
  --surface-1: #171a1d;
  --surface-2: #1d2125;
  --text-1: #f4eee7;
  --text-2: #a8a29b;
}
```

**Step 4: Run tests to verify pass**

Run: `DATABASE_URL=postgres://postgres:postgres@localhost/erp_koten cargo test cadunico -- --nocapture`
Expected: PASS for new + existing route tests.

**Step 5: Commit**

```bash
git add assets/styles/app.css templates/layouts/app.html templates/cadunico/index.html templates/cadunico/create.html tests/cadunico_routes.rs
git commit -m "style: standardize shared minimalist shell primitives"
```

### Task 7: Full Verification and UX Regression Sweep

**Files:**
- Modify (if needed from fixes): `assets/styles/app.css`, `assets/scripts/cadunico-list.js`, `assets/scripts/cadunico-form.js`, `templates/cadunico/*.html`, `tests/cadunico_routes.rs`
- Test: `assets/scripts/cadunico-list.test.js`, `assets/scripts/cadunico-form.test.js`, `tests/cadunico_routes.rs`

**Step 1: Run frontend unit tests**

Run: `bun x vitest run assets/scripts/cadunico-list.test.js assets/scripts/cadunico-form.test.js`
Expected: PASS.

**Step 2: Run backend CadUnico tests**

Run: `DATABASE_URL=postgres://postgres:postgres@localhost/erp_koten cargo test cadunico -- --nocapture`
Expected: PASS.

**Step 3: Run manual MCP UX checklist**

```text
- /cadunico: row focus no visual jump
- Enter on row opens menu
- Arrow keys navigate menu items (not table)
- Enter confirms focused menu action
- Escape closes menu/modal and restores focus
- delete dialog matches new minimalist style
- /cadunico/criar styles fully applied in desktop/mobile
```

Expected: all checks pass with no blocking issue.

**Step 4: Commit final polish**

```bash
git add assets/styles/app.css assets/scripts/cadunico-list.js assets/scripts/cadunico-form.js templates/cadunico tests/cadunico_routes.rs
git commit -m "fix: complete cadunico minimalist ux standardization"
```

