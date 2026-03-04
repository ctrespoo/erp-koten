# CadUnico List Fixes Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `/cadunico` load real records on first render, keep the list stable at 8 items per page, and fix row focus, floating actions, delete, and `Ctrl+N` behavior without rewriting the screen as a SPA.

**Architecture:** Keep the page server-rendered with Askama + HTMX. Unify initial load and fragment load through the same backend list service, move persistent interactive UI outside the HTMX-swapped fragment, and keep JavaScript focused on keyboard navigation, anchored popover behavior, and delete orchestration.

**Tech Stack:** Rust, Axum, Askama, SQLx, HTMX, vanilla JavaScript, CSS, Vitest, cargo test

---

### Task 1: Fix server-side initial load and fixed page size

**Files:**
- Modify: `tests/cadunico_routes.rs`
- Modify: `src/modules/cadunico/routes.rs`
- Modify: `src/modules/cadunico/service.rs`
- Modify: `src/modules/cadunico/templates.rs`

**Step 1: Write the failing tests**

Add these integration tests to `tests/cadunico_routes.rs`:

```rust
#[tokio::test]
async fn get_cadunico_index_should_render_existing_records_on_initial_load() {
    let database = test_database().await;
    insert_cadunico(&database.pool, "10000000001", "Registro Inicial", "Sao Paulo", "SP").await;
    let app = build_app(AppState::new(database.pool.clone()));

    let response = app
        .oneshot(Request::builder().uri("/cadunico").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("Registro Inicial"));
    assert!(!html.contains("Nenhum cadastro encontrado"));
}

#[tokio::test]
async fn get_cadunico_list_fragment_should_use_eight_items_as_default_page_size() {
    let database = test_database().await;
    for index in 1..=9 {
        insert_cadunico(
            &database.pool,
            &format!("1000000000{index}"),
            &format!("Registro {index}"),
            "Cidade",
            "SP",
        )
        .await;
    }
    let app = build_app(AppState::new(database.pool.clone()));

    let response = app
        .oneshot(Request::builder().uri("/cadunico/lista").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("Registro 9"));
    assert!(html.contains("Registro 2"));
    assert!(!html.contains("Registro 1"));
    assert!(html.contains("data-page-next-cursor=\"2\""));
}
```

**Step 2: Run the tests to verify they fail**

Run:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost/erp_koten cargo test get_cadunico_index_should_render_existing_records_on_initial_load -- --exact
DATABASE_URL=postgres://postgres:postgres@localhost/erp_koten cargo test get_cadunico_list_fragment_should_use_eight_items_as_default_page_size -- --exact
```

Expected:

- The initial-load test fails because `index()` renders `CadUnicoListPageView::empty()`.
- The page-size test fails because `normalized_page_size()` still defaults to `20`.

**Step 3: Write the minimal implementation**

Make these changes:

```rust
pub async fn index(
    State(state): State<AppState>,
    Query(query): Query<CadUnicoListInput>,
) -> Response {
    let page = match CadUnicoService::list(&state.db, query).await {
        Ok(page) => map_list_page(page, query.search.as_deref().unwrap_or("")),
        Err(CadUnicoServiceError::Unexpected(_)) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        Err(CadUnicoServiceError::Form(_)) => {
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    match render_html(&CadUnicoIndexTemplate { page: &page }) {
        Ok(html) => html.into_response(),
        Err(status) => status.into_response(),
    }
}

fn normalized_page_size(value: Option<i64>) -> i64 {
    value.unwrap_or(8).clamp(1, 100)
}

fn map_list_page(
    page: super::repository::CadUnicoListPage,
    search_value: &str,
) -> CadUnicoListPageView {
    CadUnicoListPageView {
        heading: "Cadastros",
        search_value: search_value.to_owned(),
        // items / cursors stay mapped as today
    }
}
```

Also remove or stop using `CadUnicoListPageView::empty()` in the `/cadunico` route so first render always uses real list data.

**Step 4: Run the tests to verify they pass**

Run:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost/erp_koten cargo test get_cadunico_index_should_render_existing_records_on_initial_load -- --exact
DATABASE_URL=postgres://postgres:postgres@localhost/erp_koten cargo test get_cadunico_list_fragment_should_use_eight_items_as_default_page_size -- --exact
```

Expected: both tests PASS.

**Step 5: Commit**

```bash
git add tests/cadunico_routes.rs src/modules/cadunico/routes.rs src/modules/cadunico/service.rs src/modules/cadunico/templates.rs
git commit -m "fix: load cadunico list on initial render"
```

### Task 2: Move persistent overlays out of the HTMX fragment

**Files:**
- Modify: `tests/cadunico_routes.rs`
- Modify: `templates/cadunico/index.html`
- Modify: `templates/cadunico/_list.html`
- Modify: `templates/cadunico/_delete_dialog.html`

**Step 1: Write the failing tests**

Add these integration tests:

```rust
#[tokio::test]
async fn get_cadunico_index_should_render_delete_dialog_in_the_shell() {
    let app = test_app();

    let response = app
        .oneshot(Request::builder().uri("/cadunico").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("data-delete-dialog"));
    assert!(html.contains("data-row-menu-popover-root"));
}

#[tokio::test]
async fn get_cadunico_list_fragment_should_not_render_delete_dialog_markup() {
    let app = test_app();

    let response = app
        .oneshot(Request::builder().uri("/cadunico/lista").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(!html.contains("data-delete-dialog"));
}
```

**Step 2: Run the tests to verify they fail**

Run:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost/erp_koten cargo test get_cadunico_index_should_render_delete_dialog_in_the_shell -- --exact
DATABASE_URL=postgres://postgres:postgres@localhost/erp_koten cargo test get_cadunico_list_fragment_should_not_render_delete_dialog_markup -- --exact
```

Expected:

- The shell test fails because the shell currently has no persistent popover root.
- The fragment test fails because `_list.html` still includes `_delete_dialog.html`.

**Step 3: Write the minimal implementation**

Update the templates so that:

```html
<!-- templates/cadunico/index.html -->
<main class="shell shell--wide cadunico-list-page" data-cadunico-list-root>
  ...
  {% include "cadunico/_list.html" %}
  <div class="row-menu-popover-root" data-row-menu-popover-root></div>
  {% include "cadunico/_delete_dialog.html" %}
</main>
```

```html
<!-- templates/cadunico/_list.html -->
<section class="cadunico-list-region" id="cadunico-list-region" data-list-region tabindex="0">
  ...
  <footer class="list-pagination">
    ...
  </footer>
</section>
```

Update each row to expose enough metadata for the popover script:

```html
<tr
  data-row
  data-row-id="{{ item.id }}"
  data-row-name="{{ item.fantasia }}"
  tabindex="-1"
>
```

Keep `_delete_dialog.html` as the single persistent dialog instance.

**Step 4: Run the tests to verify they pass**

Run:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost/erp_koten cargo test get_cadunico_index_should_render_delete_dialog_in_the_shell -- --exact
DATABASE_URL=postgres://postgres:postgres@localhost/erp_koten cargo test get_cadunico_list_fragment_should_not_render_delete_dialog_markup -- --exact
```

Expected: both tests PASS.

**Step 5: Commit**

```bash
git add tests/cadunico_routes.rs templates/cadunico/index.html templates/cadunico/_list.html templates/cadunico/_delete_dialog.html
git commit -m "fix: persist cadunico overlays outside htmx fragment"
```

### Task 3: Fix keyboard, floating actions, and delete orchestration in the frontend

**Files:**
- Modify: `assets/scripts/cadunico-list.test.js`
- Modify: `assets/scripts/cadunico-list.js`
- Modify: `assets/styles/app.css`

**Step 1: Write the failing tests**

Extend `assets/scripts/cadunico-list.test.js` with tests like these:

```javascript
it("bootstrapCadUnicoList should open a floating menu for the active row", () => {
  const root = mountList();
  const popoverRoot = root.querySelector("[data-row-menu-popover-root]");

  document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
  document.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));

  const menu = popoverRoot.querySelector("[data-row-menu-popover]");
  expect(menu.hidden).toBe(false);
  expect(menu.getAttribute("data-row-id")).toBe("10");
});

it("bootstrapCadUnicoList should use the persistent delete dialog after a list swap", () => {
  const root = mountList();
  window.htmx = { ajax: vi.fn() };

  document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
  document.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
  root.querySelector("[data-row-delete]").click();

  const dialog = root.querySelector("[data-delete-dialog]");
  expect(dialog.open).toBe(true);
});

it("bootstrapCadUnicoList should navigate to create on Ctrl+N", () => {
  const root = mountList();
  const createLink = root.querySelector("[data-create-link]");
  const clickSpy = vi.fn((event) => event.preventDefault());

  createLink.addEventListener("click", clickSpy);
  document.dispatchEvent(new KeyboardEvent("keydown", { key: "n", ctrlKey: true, bubbles: true }));

  expect(clickSpy).toHaveBeenCalledTimes(1);
});
```

Update `mountList()` so the dialog and popover root are outside `#cadunico-list-region`, matching the production shell.

**Step 2: Run the tests to verify they fail**

Run:

```bash
bun x vitest run assets/scripts/cadunico-list.test.js
```

Expected:

- The floating-menu test fails because the menu still lives inline inside the row.
- The delete test fails or throws because the code still closes over a stale dialog/list subtree.

**Step 3: Write the minimal implementation**

Refactor `assets/scripts/cadunico-list.js` around the current DOM contract:

```javascript
function currentDialog(root) {
  return root.querySelector("[data-delete-dialog]");
}

function popoverRoot(root) {
  return root.querySelector("[data-row-menu-popover-root]");
}

function openFloatingMenu(root, row) {
  const host = popoverRoot(root);
  if (!(host instanceof HTMLElement)) return;

  host.innerHTML = `
    <div class="row-menu-popover" data-row-menu-popover data-row-id="${row.dataset.rowId}">
      <button type="button" disabled aria-disabled="true">Editar (em breve)</button>
      <button type="button" data-row-delete data-row-name="${row.dataset.rowName ?? ""}">Excluir</button>
    </div>
  `;

  const menu = host.querySelector("[data-row-menu-popover]");
  menu.hidden = false;
  positionPopover(menu, row);
  menu.querySelector("[data-row-delete]")?.focus();
}

if (event.ctrlKey && key === "n") {
  event.preventDefault();
  createLink?.click();
  return;
}
```

Implementation notes:

- Stop relying on `[data-row-menu]` inside the row.
- Re-query the dialog from the live DOM each time it is needed.
- Keep one teardown function and re-bootstrap after HTMX swaps.
- Close the popover on `Escape`, outside click, page change, and successful delete.
- When delete is confirmed, keep the current search query in the DELETE refresh request.

Update `assets/styles/app.css` in the same task to support:

```css
.cadunico-list-table tbody tr[data-row-active="true"] td {
  background: transparent;
}

.cadunico-list-table tbody tr[data-row-active="true"] td:first-child > * ,
.cadunico-list-table tbody tr[data-row-active="true"] td {
  position: relative;
}

.row-menu-popover-root {
  position: fixed;
  inset: 0;
  pointer-events: none;
}

.row-menu-popover {
  position: absolute;
  pointer-events: auto;
}
```

The exact CSS can differ, but the active-row highlight must be visually contained and the menu must float above the table instead of reflowing it.

**Step 4: Run the tests to verify they pass**

Run:

```bash
bun x vitest run assets/scripts/cadunico-list.test.js
```

Expected: all cadunico list tests PASS.

**Step 5: Commit**

```bash
git add assets/scripts/cadunico-list.test.js assets/scripts/cadunico-list.js assets/styles/app.css
git commit -m "fix: stabilize cadunico list interactions"
```

### Task 4: Verify the full flow end-to-end

**Files:**
- Review only: `tests/cadunico_routes.rs`
- Review only: `assets/scripts/cadunico-list.test.js`

**Step 1: Run the relevant automated suites**

Run:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost/erp_koten cargo test --test cadunico_routes
bun x vitest run assets/scripts/cadunico-list.test.js
```

Expected: PASS.

**Step 2: Run quick manual verification in the browser**

Run the app, then verify `/cadunico` with at least 9 records in the database:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost/erp_koten cargo run
```

Manual checks:

- opening `/cadunico` already shows records
- only 8 rows appear on the first page
- `ArrowDown` / `ArrowUp` move focus between rows
- the active-row highlight stays contained
- `Enter` opens a floating menu aligned with the row actions
- `Escape` closes the menu and restores row focus
- `Ctrl+N` navigates to `/cadunico/criar`
- delete removes the chosen item and refreshes the list correctly

**Step 3: If manual verification uncovers a small mismatch, fix it immediately and rerun the affected test suite**

Run:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost/erp_koten cargo test --test cadunico_routes
bun x vitest run assets/scripts/cadunico-list.test.js
```

Expected: still PASS after any final adjustment.

**Step 4: Commit the final polish**

```bash
git add tests/cadunico_routes.rs assets/scripts/cadunico-list.test.js assets/scripts/cadunico-list.js assets/styles/app.css templates/cadunico/index.html templates/cadunico/_list.html templates/cadunico/_delete_dialog.html src/modules/cadunico/routes.rs src/modules/cadunico/service.rs src/modules/cadunico/templates.rs
git commit -m "fix: complete cadunico list regression cleanup"
```
