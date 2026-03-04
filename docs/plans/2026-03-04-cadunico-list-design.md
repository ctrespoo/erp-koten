# CadUnico List Design

**Date:** 2026-03-04

**Goal:** Build the `/cadunico` list screen with keyboard-first navigation, debounced search, cursor pagination, and row actions that fit the current Askama + HTMX stack.

## Context

The current CadUnico module already supports rendering the create form, validating input, and persisting records into PostgreSQL. The `/cadunico` route still renders only a placeholder shell with a link to create a new record.

The new list screen needs to behave like an operational console. Users must be able to search, paginate, select rows, open row actions, and delete entries without touching the mouse. The page must fit inside the viewport and keep vertical scrolling contained to the results area so keyboard users never need browser scroll interaction.

## Decisions

### UI architecture

- Keep the existing server-rendered stack: Askama for templates and HTMX for partial updates.
- Render `/cadunico` as the full page shell.
- Render the table region through a reusable partial so search, pagination, and deletion can swap only the results area.
- Preserve the current dark utilitarian visual direction, but reframe the page as a dense, keyboard-driven console rather than a landing page.

### Search and pagination

- Add a search field with a 300 ms debounce.
- Search across `cpf_cnpj`, `fantasia`, and `cidade`.
- Reset pagination whenever the search query changes.
- Implement cursor pagination using `id DESC` as the stable ordering.
- Support forward and backward navigation through `before` and `after` cursor parameters while keeping the rendered order visually consistent.

### Keyboard model

- `Ctrl+K` focuses the search field.
- `Esc` exits the search field, closes the row menu, or closes the delete confirmation dialog depending on the current active layer.
- `Ctrl+N` navigates to `/cadunico/criar`.
- `ArrowUp` and `ArrowDown` move the active row selection.
- `ArrowLeft` and `ArrowRight` change pages when the focus is outside the search field.
- `Enter` on an active row opens an action menu anchored to that row.
- The row menu is fully keyboard navigable and includes:
  - `Editar`, visible but disabled because it is not implemented yet.
  - `Excluir`, available and guarded by a confirmation dialog.

### Accessibility and focus management

- Use a roving tabindex model for table row selection so one row is active at a time.
- Move focus into the row action menu when it opens, then restore focus to the originating row when it closes.
- Keep the page height within `100vh`.
- Constrain overflow to the table body area and auto-scroll the active row into view as keyboard navigation moves through results.
- Avoid intercepting unrelated browser shortcuts; only capture the shortcuts needed for the workflow.

### Backend structure

- Extend the repository with a cursor-based list query and a delete operation.
- Add service-layer request parsing and typed errors for list and delete flows.
- Keep error handling typed with `thiserror` and `Result`, following the existing module style.
- Reuse server-rendered fragments for the table, toolbar state, and empty state.

### Testing strategy

- Add route tests for:
  - list page rendering
  - search filtering
  - cursor pagination
  - delete flow
- Add frontend tests for:
  - debounced search
  - keyboard row navigation
  - page navigation with arrow keys
  - row action menu open/close behavior
  - `Ctrl+K`, `Ctrl+N`, and `Esc`
- If the existing setup supports it cleanly, add an end-to-end test covering search, row navigation, menu open, and delete.

## Data flow

1. The browser loads `GET /cadunico`, which renders the full list shell and the first page of results.
2. Typing in search triggers a debounced HTMX request to refresh the results fragment with the current query and reset cursor state.
3. Pagination controls and left/right arrow shortcuts request the previous or next cursor page through the same fragment route.
4. Keyboard navigation updates the active row client-side without changing browser focus outside the list region.
5. Pressing `Enter` on a selected row opens the row action menu.
6. Confirming delete sends a request to remove the record, then re-renders the results fragment with the current search and cursor context.

## Files expected to change

- `assets/scripts/cadunico-list.js`
- `assets/styles/app.css`
- `src/modules/cadunico/mod.rs`
- `src/modules/cadunico/repository.rs`
- `src/modules/cadunico/routes.rs`
- `src/modules/cadunico/service.rs`
- `src/modules/cadunico/templates.rs`
- `templates/cadunico/index.html`
- `templates/cadunico/_list.html`
- `templates/cadunico/_delete_dialog.html`
- `tests/cadunico_routes.rs`
- `assets/scripts/cadunico-list.test.js`
