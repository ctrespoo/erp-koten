# CadUnico Persistence Design

**Date:** 2026-03-04

**Goal:** Persist the CadUnico form into PostgreSQL with `sqlx`, replacing the current validate-only submit flow.

## Context

The current Rust module renders the full CadUnico form and validates a small normalized subset of the payload. A successful `POST /cadunico` only returns `HX-Redirect: /cadunico`; nothing is stored yet.

The legacy Go schema is the reference for field coverage, but the new Rust implementation does not need table compatibility with the old application. We can create a new PostgreSQL table that preserves the same business fields.

## Decisions

### Storage model

- Create a new PostgreSQL table named `cadunico`.
- Use `bigserial` primary key.
- Keep `cpf_cnpj` unique.
- Store `telefones` as `text[]`.
- Store `aniversario` as `date`.
- Store `created_at` and `updated_at` as `timestamptz` with `now()` defaults.

### Application architecture

- Add `AppState` with a shared `PgPool`.
- Change router construction to accept application state.
- Keep normalization and validation inside the CadUnico service layer.
- Add a repository layer to isolate SQL from validation and HTTP concerns.

### Form normalization

- Normalize required string fields with `trim()`.
- Convert empty optional strings to `None`.
- Strip non-digits from `cpf_cnpj`, `cep`, `cep_cobranca`, and `telefones`.
- Uppercase `uf` and `uf_cobranca`.
- Parse `aniversario` into `Option<NaiveDate>`.
- Preserve boolean flags as submitted by the form.

### Error handling

- Validation failures remain `422 Unprocessable Entity` and render the existing modal fragment.
- Unique violation on `cpf_cnpj` becomes a user-facing `422` with `cpf_cnpj` marked invalid.
- Unexpected database failures return `500 Internal Server Error`.
- Use `thiserror` for typed service and repository errors; do not use `unwrap` in production code.

### Testing strategy

- Extend unit tests around normalization to cover optional fields, digit-only fields, and date parsing.
- Extend service tests to cover the new normalized shape and duplicate-key mapping.
- Add integration tests for `POST /cadunico` success, invalid payload, single phone input, and duplicate `cpf_cnpj`.
- Run integration tests against the PostgreSQL database configured by `.env`.

## Data flow

1. Axum receives `POST /cadunico`.
2. The route deserializes `CadUnicoFormInput`.
3. The service normalizes and validates the payload.
4. The repository inserts the normalized record through `sqlx`.
5. The route returns `HX-Redirect: /cadunico` on success.
6. Validation and duplicate-key failures render the modal fragment with field-level feedback.

## Files expected to change

- `Cargo.toml`
- `src/app.rs`
- `src/lib.rs`
- `src/main.rs`
- `src/modules/cadunico/errors.rs`
- `src/modules/cadunico/forms.rs`
- `src/modules/cadunico/mod.rs`
- `src/modules/cadunico/routes.rs`
- `src/modules/cadunico/service.rs`
- `src/state.rs`
- `tests/cadunico_routes.rs`
- `migrations/*`

