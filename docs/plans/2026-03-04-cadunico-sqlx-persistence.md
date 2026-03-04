# CadUnico SQLx Persistence Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Persist the full CadUnico form into PostgreSQL through `sqlx` while preserving the existing HTMX validation UX.

**Architecture:** The router will receive an application state containing a shared `PgPool`. The CadUnico service will own normalization and validation, and a repository module will own SQL insert logic plus duplicate-key detection.

**Tech Stack:** Rust, Axum, Askama, Tokio, SQLx, PostgreSQL, ThisError

---

### Task 1: Add database-backed route coverage first

**Files:**
- Modify: `tests/cadunico_routes.rs`
- Modify: `Cargo.toml`

**Step 1: Write the failing tests**

```rust
#[tokio::test]
async fn post_cadunico_should_persist_full_payload_when_valid() { /* ... */ }

#[tokio::test]
async fn post_cadunico_should_return_modal_fragment_when_cpf_cnpj_is_duplicated() { /* ... */ }
```

**Step 2: Run test to verify it fails**

Run: `cargo test post_cadunico_should_persist_full_payload_when_valid post_cadunico_should_return_modal_fragment_when_cpf_cnpj_is_duplicated`
Expected: FAIL because the app has no database state or persistence path yet.

**Step 3: Write minimal implementation scaffolding**

```rust
pub struct AppState {
    pub db: PgPool,
}
```

**Step 4: Run test to verify it still fails for the expected missing persistence behavior**

Run: `cargo test post_cadunico_should_persist_full_payload_when_valid post_cadunico_should_return_modal_fragment_when_cpf_cnpj_is_duplicated`
Expected: FAIL in application code, not because of syntax or missing imports.

**Step 5: Commit**

```bash
git add Cargo.toml tests/cadunico_routes.rs src/state.rs src/app.rs src/main.rs src/lib.rs
git commit -m "test: cover cadunico database persistence flow"
```

### Task 2: Expand normalization and validation to the full CadUnico payload

**Files:**
- Modify: `src/modules/cadunico/forms.rs`
- Modify: `src/modules/cadunico/service.rs`
- Modify: `src/modules/cadunico/errors.rs`

**Step 1: Write the failing tests**

```rust
#[test]
fn normalize_should_convert_blank_optional_fields_to_none() { /* ... */ }

#[test]
fn normalize_should_parse_aniversario_when_present() { /* ... */ }
```

**Step 2: Run test to verify it fails**

Run: `cargo test normalize_should_convert_blank_optional_fields_to_none normalize_should_parse_aniversario_when_present`
Expected: FAIL because the normalized struct does not carry the full persistence payload yet.

**Step 3: Write minimal implementation**

```rust
pub struct CadUnicoFormData {
    pub cpf_cnpj: String,
    pub inscricao_estadual: Option<String>,
    pub inscricao_municipal: Option<String>,
    // ...
    pub updated_at: Option<OffsetDateTime>,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test normalize_should_convert_blank_optional_fields_to_none normalize_should_parse_aniversario_when_present`
Expected: PASS

**Step 5: Commit**

```bash
git add src/modules/cadunico/forms.rs src/modules/cadunico/service.rs src/modules/cadunico/errors.rs
git commit -m "feat: normalize full cadunico payload"
```

### Task 3: Create the SQL schema and repository insertion path

**Files:**
- Create: `migrations/20260304130000_create_cadunico.sql`
- Create: `src/modules/cadunico/repository.rs`
- Modify: `src/modules/cadunico/mod.rs`
- Modify: `src/modules/cadunico/service.rs`

**Step 1: Write the failing tests**

```rust
#[tokio::test]
async fn create_should_return_duplicate_error_when_cpf_cnpj_already_exists() { /* ... */ }
```

**Step 2: Run test to verify it fails**

Run: `cargo test create_should_return_duplicate_error_when_cpf_cnpj_already_exists`
Expected: FAIL because there is no migration or repository insert path yet.

**Step 3: Write minimal implementation**

```rust
sqlx::query(
    r#"
    INSERT INTO cadunico (cpf_cnpj, fantasia, ...)
    VALUES ($1, $2, ...)
    "#
)
```

**Step 4: Run test to verify it passes**

Run: `cargo test create_should_return_duplicate_error_when_cpf_cnpj_already_exists`
Expected: PASS

**Step 5: Commit**

```bash
git add migrations src/modules/cadunico/repository.rs src/modules/cadunico/mod.rs src/modules/cadunico/service.rs
git commit -m "feat: persist cadunico with sqlx"
```

### Task 4: Wire the route and application state to the repository-backed service

**Files:**
- Modify: `src/app.rs`
- Modify: `src/main.rs`
- Modify: `src/modules/cadunico/routes.rs`
- Modify: `tests/cadunico_routes.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn post_cadunico_should_return_hx_redirect_when_payload_is_persisted() { /* ... */ }
```

**Step 2: Run test to verify it fails**

Run: `cargo test post_cadunico_should_return_hx_redirect_when_payload_is_persisted`
Expected: FAIL because the route still only validates the payload.

**Step 3: Write minimal implementation**

```rust
pub async fn submit(
    State(state): State<AppState>,
    form: Result<Form<CadUnicoFormInput>, FormRejection>,
) -> Response {
    // validate -> insert -> redirect
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test post_cadunico_should_return_hx_redirect_when_payload_is_persisted`
Expected: PASS

**Step 5: Commit**

```bash
git add src/app.rs src/main.rs src/modules/cadunico/routes.rs tests/cadunico_routes.rs
git commit -m "feat: wire cadunico submit to postgres"
```

### Task 5: Run the full verification pass and clean up

**Files:**
- Modify: `src/modules/cadunico/*`
- Modify: `tests/cadunico_routes.rs`

**Step 1: Run focused tests**

Run: `cargo test cadunico`
Expected: PASS

**Step 2: Run the full test suite**

Run: `cargo test`
Expected: PASS

**Step 3: Refactor only if tests remain green**

```rust
fn normalize_optional(value: Option<String>) -> Option<String> { /* ... */ }
```

**Step 4: Run the full test suite again**

Run: `cargo test`
Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml src tests migrations docs/plans
git commit -m "feat: persist cadunico forms in postgres"
```
