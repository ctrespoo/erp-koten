# Cadastro Unico Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the first `Cadastro Unico` slice with a full tabbed form at `/cadunico/criar`, keyboard-first navigation, frontend masks, HTMX submit, backend validation without persistence, modal error handling, and success redirect to `/cadunico`.

**Architecture:** Convert the crate into a testable Axum application with a small library surface, Askama templates for pages and fragments, and a focused `cadunico` module for routes, forms, validation, templates, and error mapping. Keep the first slice persistence-free: normalize and validate on the backend, use HTMX for form submission and modal updates, and isolate keyboard and mask behavior inside one small browser script.

**Tech Stack:** Rust 2024, Axum, Tokio, Askama, HTMX, SQLX (declared for the stack, not used yet), dotenvy, tower-http, thiserror, Vitest + jsdom

---

## Preflight

- Use `@using-git-worktrees` before executing this plan.
- This repository currently has no `.worktrees/` or `worktrees/` directory, so ask the user where worktrees should live before creating one.
- Execute every coding task with `@test-driven-development`.
- Apply `@rust-best-practices` for ownership, error handling, and test naming.
- Before claiming completion, run `@verification-before-completion`.

### Task 1: Bootstrap a Testable Axum App

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/main.rs`
- Create: `src/lib.rs`
- Create: `src/app.rs`
- Create: `src/modules/mod.rs`
- Create: `src/modules/cadunico/mod.rs`
- Create: `src/modules/cadunico/routes.rs`
- Test: `tests/cadunico_routes.rs`

**Step 1: Write the failing test**

```rust
use axum::body::Body;
use axum::http::{Request, StatusCode};
use erp_koten::app::build_app;
use tower::ServiceExt;

#[tokio::test]
async fn get_cadunico_index_should_return_ok() {
    let app = build_app();

    let response = app
        .oneshot(Request::builder().uri("/cadunico").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn get_cadunico_create_should_return_ok() {
    let app = build_app();

    let response = app
        .oneshot(Request::builder().uri("/cadunico/criar").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cadunico_routes get_cadunico_index_should_return_ok -- --exact`
Expected: FAIL with unresolved crate items such as `erp_koten::app` or missing dependencies.

**Step 3: Write minimal implementation**

```toml
[dependencies]
anyhow = "1"
askama = { version = "0.12", features = ["with-axum"] }
axum = { version = "0.8", features = ["form", "macros"] }
dotenvy = "0.15"
serde = { version = "1", features = ["derive"] }
sqlx = { version = "0.8", default-features = false, features = ["runtime-tokio-rustls", "postgres"] }
thiserror = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
tower = { version = "0.5", features = ["util"] }
tower-http = { version = "0.6", features = ["fs"] }

[dev-dependencies]
tower = { version = "0.5", features = ["util"] }
```

```rust
// src/lib.rs
pub mod app;
pub mod modules;
```

```rust
// src/modules/mod.rs
pub mod cadunico;
```

```rust
// src/modules/cadunico/mod.rs
pub mod routes;
```

```rust
// src/app.rs
use axum::{routing::get, Router};

use crate::modules::cadunico::routes;

pub fn build_app() -> Router {
    Router::new()
        .route("/", get(routes::home))
        .route("/cadunico", get(routes::index))
        .route("/cadunico/criar", get(routes::create))
}
```

```rust
// src/modules/cadunico/routes.rs
use axum::response::Html;

pub async fn home() -> Html<&'static str> {
    Html("<h1>ERP Koten</h1>")
}

pub async fn index() -> Html<&'static str> {
    Html("<h1>Cadastro Unico</h1>")
}

pub async fn create() -> Html<&'static str> {
    Html("<h1>Novo Cadastro Unico</h1>")
}
```

```rust
// src/main.rs
use anyhow::Result;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, erp_koten::app::build_app()).await?;

    Ok(())
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test cadunico_routes -- --nocapture`
Expected: PASS with `2 passed`.

**Step 5: Commit**

```bash
git add Cargo.toml src/main.rs src/lib.rs src/app.rs src/modules/mod.rs src/modules/cadunico/mod.rs src/modules/cadunico/routes.rs tests/cadunico_routes.rs
git commit -m "feat: bootstrap cadunico routes"
```

### Task 2: Add Askama Layout, Static Assets, and the `/cadunico` Placeholder Page

**Files:**
- Modify: `src/app.rs`
- Modify: `src/modules/cadunico/routes.rs`
- Create: `src/modules/cadunico/templates.rs`
- Create: `templates/layouts/app.html`
- Create: `templates/cadunico/index.html`
- Create: `assets/styles/app.css`
- Test: `tests/cadunico_routes.rs`

**Step 1: Write the failing test**

```rust
use axum::body::{to_bytes, Body};
use axum::http::Request;
use erp_koten::app::build_app;
use tower::ServiceExt;

#[tokio::test]
async fn get_cadunico_index_should_render_html_layout() {
    let app = build_app();

    let response = app
        .oneshot(Request::builder().uri("/cadunico").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("Cadastro Unico"));
    assert!(html.contains("/assets/styles/app.css"));
    assert!(html.contains("/cadunico/criar"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cadunico_routes get_cadunico_index_should_render_html_layout -- --exact`
Expected: FAIL because the route still returns plain inline HTML and no asset link.

**Step 3: Write minimal implementation**

```rust
// src/modules/cadunico/templates.rs
use askama::Template;

#[derive(Template)]
#[template(path = "cadunico/index.html")]
pub struct CadUnicoIndexTemplate;
```

```rust
// src/modules/cadunico/routes.rs
use axum::response::Html;

use super::templates::CadUnicoIndexTemplate;

pub async fn index() -> Html<String> {
    Html(CadUnicoIndexTemplate.render().unwrap())
}
```

```rust
// src/app.rs
use tower_http::services::ServeDir;

pub fn build_app() -> Router {
    Router::new()
        .route("/", get(routes::home))
        .route("/cadunico", get(routes::index))
        .route("/cadunico/criar", get(routes::create))
        .nest_service("/assets", ServeDir::new("assets"))
}
```

```html
<!-- templates/layouts/app.html -->
<!DOCTYPE html>
<html lang="pt-BR">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>{% block title %}ERP Koten{% endblock %}</title>
    <link rel="stylesheet" href="/assets/styles/app.css" />
    <script src="https://unpkg.com/htmx.org@2.0.4" defer></script>
  </head>
  <body>
    {% block body %}{% endblock %}
  </body>
</html>
```

```html
<!-- templates/cadunico/index.html -->
{% extends "layouts/app.html" %}

{% block title %}Cadastro Unico{% endblock %}

{% block body %}
<main class="shell shell--narrow">
  <header class="page-header">
    <p class="eyebrow">Modulo interno</p>
    <h1>Cadastro Unico</h1>
    <a class="primary-link" href="/cadunico/criar">Criar cadastro</a>
  </header>
</main>
{% endblock %}
```

```css
/* assets/styles/app.css */
:root {
  --bg: #f4efe7;
  --panel: #fffdfa;
  --ink: #161413;
  --muted: #6a625b;
  --line: #d7cfc5;
  --accent: #184a45;
  --accent-strong: #103430;
  --focus: #c14f2b;
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test cadunico_routes get_cadunico_index_should_render_html_layout -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/app.rs src/modules/cadunico/routes.rs src/modules/cadunico/templates.rs templates/layouts/app.html templates/cadunico/index.html assets/styles/app.css tests/cadunico_routes.rs
git commit -m "feat: add cadunico layout and list page"
```

### Task 3: Render the Full Tabbed Form Shell at `/cadunico/criar`

**Files:**
- Modify: `src/modules/cadunico/routes.rs`
- Modify: `src/modules/cadunico/templates.rs`
- Create: `templates/cadunico/create.html`
- Test: `tests/cadunico_routes.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn get_cadunico_create_should_render_tabbed_form_and_all_schema_fields() {
    let app = build_app();

    let response = app
        .oneshot(Request::builder().uri("/cadunico/criar").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("data-tab-list"));
    assert!(html.contains("dados-principais"));
    assert!(html.contains("cobranca"));
    assert!(html.contains("name=\"cpf_cnpj\""));
    assert!(html.contains("name=\"fantasia\""));
    assert!(html.contains("name=\"codigo_ibge\""));
    assert!(html.contains("name=\"enviar_nfe\""));
    assert!(html.contains("name=\"codigo_ibge_cobranca\""));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cadunico_routes get_cadunico_create_should_render_tabbed_form_and_all_schema_fields -- --exact`
Expected: FAIL because `/cadunico/criar` still returns a placeholder heading.

**Step 3: Write minimal implementation**

```rust
// src/modules/cadunico/templates.rs
use askama::Template;

pub struct TabView<'a> {
    pub id: &'a str,
    pub label: &'a str,
}

#[derive(Template)]
#[template(path = "cadunico/create.html")]
pub struct CadUnicoCreateTemplate<'a> {
    pub tabs: &'a [TabView<'a>],
}

pub const TABS: &[TabView<'static>] = &[
    TabView { id: "dados-principais", label: "Dados principais" },
    TabView { id: "endereco", label: "Endereco" },
    TabView { id: "parametros", label: "Parametros" },
    TabView { id: "cobranca", label: "Cobranca" },
];
```

```rust
// src/modules/cadunico/routes.rs
pub async fn create() -> Html<String> {
    Html(CadUnicoCreateTemplate { tabs: TABS }.render().unwrap())
}
```

```html
<!-- templates/cadunico/create.html -->
{% extends "layouts/app.html" %}

{% block title %}Novo Cadastro Unico{% endblock %}

{% block body %}
<main class="shell shell--wide" data-cadunico-root>
  <header class="page-header">
    <div>
      <p class="eyebrow">Cadastro Unico</p>
      <h1>Novo cadastro</h1>
    </div>
    <p class="shortcut-help">
      Tab/Shift+Tab ou setas verticais navegam campos. Ctrl + setas horizontais trocam tabs. Ctrl+S envia.
    </p>
  </header>

  <form
    class="cadunico-form"
    hx-post="/cadunico"
    hx-target="#cadunico-modal-root"
    hx-swap="innerHTML"
    data-cadunico-form
  >
    <nav class="tab-strip" aria-label="Secoes do cadastro" data-tab-list>
      {% for tab in tabs %}
      <button type="button" class="tab-button" data-tab-trigger data-tab-id="{{ tab.id }}">
        {{ tab.label }}
      </button>
      {% endfor %}
    </nav>

    <section class="tab-panel is-active" data-tab-panel="dados-principais">
      <input name="cpf_cnpj" data-mask="cpf_cnpj" required />
      <input name="inscricao_estadual" />
      <input name="inscricao_municipal" />
      <input name="fantasia" required />
      <input name="inss" />
      <input name="crea" />
      <input name="email" type="email" />
      <div data-phones-list>
        <input name="telefones" data-mask="telefone" />
      </div>
      <input name="aniversario" type="date" />
      <input name="id_estrangeiro" />
      <input name="codigo_pais" />
    </section>

    <section class="tab-panel" data-tab-panel="endereco">
      <input name="cep" data-mask="cep" required />
      <input name="endereco" required />
      <input name="numero" />
      <input name="complemento" />
      <input name="bairro" required />
      <input name="cidade" required />
      <input name="uf" maxlength="2" required />
      <input name="codigo_ibge" maxlength="7" required />
    </section>

    <section class="tab-panel" data-tab-panel="parametros">
      <input type="checkbox" name="enviar_nfe" />
      <input type="checkbox" name="enviar_boleto" />
      <input type="checkbox" name="enviar_extrato" />
      <input type="checkbox" name="etiqueta" />
      <input type="checkbox" name="comissao" />
      <input type="checkbox" name="construcao_civil" />
    </section>

    <section class="tab-panel" data-tab-panel="cobranca">
      <input name="cep_cobranca" data-mask="cep" />
      <input name="endereco_cobranca" />
      <input name="numero_cobranca" />
      <input name="complemento_cobranca" />
      <input name="bairro_cobranca" />
      <input name="cidade_cobranca" />
      <input name="uf_cobranca" maxlength="2" />
      <input name="codigo_ibge_cobranca" maxlength="7" />
      <input name="referencia_cobranca" />
    </section>

    <footer class="form-footer">
      <button type="submit">Salvar</button>
    </footer>
  </form>

  <div id="cadunico-modal-root" aria-live="assertive"></div>
</main>
{% endblock %}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test cadunico_routes get_cadunico_create_should_render_tabbed_form_and_all_schema_fields -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/modules/cadunico/routes.rs src/modules/cadunico/templates.rs templates/cadunico/create.html tests/cadunico_routes.rs
git commit -m "feat: render cadunico tabbed form shell"
```

### Task 4: Add Form Input Types, Normalization, and Validation Rules

**Files:**
- Create: `src/modules/cadunico/forms.rs`
- Create: `src/modules/cadunico/service.rs`
- Create: `src/modules/cadunico/errors.rs`
- Modify: `src/modules/cadunico/mod.rs`
- Test: `src/modules/cadunico/forms.rs`
- Test: `src/modules/cadunico/service.rs`

**Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod normalize {
    use super::CadUnicoFormInput;

    #[test]
    fn normalize_should_strip_non_digits_from_cpf_cnpj() {
        let input = CadUnicoFormInput {
            cpf_cnpj: "123.456.789-01".into(),
            ..CadUnicoFormInput::minimal_valid()
        };

        let normalized = input.normalize();

        assert_eq!(normalized.cpf_cnpj, "12345678901");
    }

    #[test]
    fn normalize_should_uppercase_uf_fields() {
        let input = CadUnicoFormInput {
            uf: "sp".into(),
            uf_cobranca: Some("rj".into()),
            ..CadUnicoFormInput::minimal_valid()
        };

        let normalized = input.normalize();

        assert_eq!(normalized.uf, "SP");
        assert_eq!(normalized.uf_cobranca.as_deref(), Some("RJ"));
    }
}

#[cfg(test)]
mod validate {
    use super::{CadUnicoFormInput, CadUnicoService};

    #[test]
    fn validate_should_return_error_when_required_fields_are_blank() {
        let input = CadUnicoFormInput::default();

        let error = CadUnicoService::validate(input).unwrap_err();

        assert_eq!(error.to_string(), "cpf_cnpj is required");
    }

    #[test]
    fn validate_should_accept_minimal_valid_payload() {
        let input = CadUnicoFormInput::minimal_valid();

        let result = CadUnicoService::validate(input);

        assert!(result.is_ok(), "unexpected validation error: {:?}", result.unwrap_err());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test normalize_should_strip_non_digits_from_cpf_cnpj -- --exact`
Expected: FAIL because `forms.rs`, `service.rs`, and `CadUnicoService` do not exist yet.

**Step 3: Write minimal implementation**

```rust
// src/modules/cadunico/forms.rs
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct CadUnicoFormInput {
    pub cpf_cnpj: String,
    pub inscricao_estadual: Option<String>,
    pub inscricao_municipal: Option<String>,
    pub fantasia: String,
    pub inss: Option<String>,
    pub crea: Option<String>,
    pub email: Option<String>,
    #[serde(default)]
    pub telefones: Vec<String>,
    pub aniversario: Option<String>,
    pub id_estrangeiro: Option<String>,
    pub codigo_pais: Option<String>,
    pub cep: String,
    pub endereco: String,
    pub numero: Option<String>,
    pub complemento: Option<String>,
    pub bairro: String,
    pub cidade: String,
    pub uf: String,
    pub codigo_ibge: String,
    #[serde(default)]
    pub enviar_nfe: bool,
    #[serde(default)]
    pub enviar_boleto: bool,
    #[serde(default)]
    pub enviar_extrato: bool,
    #[serde(default)]
    pub etiqueta: bool,
    #[serde(default)]
    pub comissao: bool,
    #[serde(default)]
    pub construcao_civil: bool,
    pub cep_cobranca: Option<String>,
    pub endereco_cobranca: Option<String>,
    pub numero_cobranca: Option<String>,
    pub complemento_cobranca: Option<String>,
    pub bairro_cobranca: Option<String>,
    pub cidade_cobranca: Option<String>,
    pub uf_cobranca: Option<String>,
    pub codigo_ibge_cobranca: Option<String>,
    pub referencia_cobranca: Option<String>,
}

#[derive(Debug)]
pub struct CadUnicoFormData {
    pub cpf_cnpj: String,
    pub cep: String,
    pub uf: String,
    pub uf_cobranca: Option<String>,
    pub telefones: Vec<String>,
    pub raw: CadUnicoFormInput,
}

fn digits_only(value: &str) -> String {
    value.chars().filter(|ch| ch.is_ascii_digit()).collect()
}

impl CadUnicoFormInput {
    pub fn minimal_valid() -> Self {
        Self {
            cpf_cnpj: "12345678901".into(),
            fantasia: "Cliente teste".into(),
            cep: "01001000".into(),
            endereco: "Rua A".into(),
            bairro: "Centro".into(),
            cidade: "Sao Paulo".into(),
            uf: "SP".into(),
            codigo_ibge: "3550308".into(),
            ..Self::default()
        }
    }

    pub fn normalize(self) -> CadUnicoFormData {
        CadUnicoFormData {
            cpf_cnpj: digits_only(&self.cpf_cnpj),
            cep: digits_only(&self.cep),
            uf: self.uf.trim().to_uppercase(),
            uf_cobranca: self.uf_cobranca.map(|value| value.trim().to_uppercase()),
            telefones: self
                .telefones
                .into_iter()
                .map(|value| digits_only(&value))
                .filter(|value| !value.is_empty())
                .collect(),
            raw: self,
        }
    }
}
```

```rust
// src/modules/cadunico/errors.rs
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CadUnicoFormError {
    #[error("cpf_cnpj is required")]
    MissingCpfCnpj,
    #[error("fantasia is required")]
    MissingFantasia,
    #[error("cep is required")]
    MissingCep,
    #[error("endereco is required")]
    MissingEndereco,
    #[error("bairro is required")]
    MissingBairro,
    #[error("cidade is required")]
    MissingCidade,
    #[error("uf is required")]
    MissingUf,
    #[error("codigo_ibge is required")]
    MissingCodigoIbge,
}
```

```rust
// src/modules/cadunico/service.rs
use super::errors::CadUnicoFormError;
use super::forms::{CadUnicoFormData, CadUnicoFormInput};

pub struct CadUnicoService;

impl CadUnicoService {
    pub fn validate(input: CadUnicoFormInput) -> Result<CadUnicoFormData, CadUnicoFormError> {
        let normalized = input.normalize();

        if normalized.cpf_cnpj.is_empty() {
            return Err(CadUnicoFormError::MissingCpfCnpj);
        }
        if normalized.raw.fantasia.trim().is_empty() {
            return Err(CadUnicoFormError::MissingFantasia);
        }
        if normalized.cep.is_empty() {
            return Err(CadUnicoFormError::MissingCep);
        }
        if normalized.raw.endereco.trim().is_empty() {
            return Err(CadUnicoFormError::MissingEndereco);
        }
        if normalized.raw.bairro.trim().is_empty() {
            return Err(CadUnicoFormError::MissingBairro);
        }
        if normalized.raw.cidade.trim().is_empty() {
            return Err(CadUnicoFormError::MissingCidade);
        }
        if normalized.uf.is_empty() {
            return Err(CadUnicoFormError::MissingUf);
        }
        if normalized.raw.codigo_ibge.trim().is_empty() {
            return Err(CadUnicoFormError::MissingCodigoIbge);
        }

        Ok(normalized)
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test normalize_should_strip_non_digits_from_cpf_cnpj -- --exact`
Expected: PASS.

Run: `cargo test validate_should_return_error_when_required_fields_are_blank -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/modules/cadunico/forms.rs src/modules/cadunico/service.rs src/modules/cadunico/errors.rs src/modules/cadunico/mod.rs
git commit -m "feat: add cadunico form normalization and validation"
```

### Task 5: Add `POST /cadunico` with HTMX Success Redirect and Backend Error Modal Contract

**Files:**
- Modify: `src/app.rs`
- Modify: `src/modules/cadunico/routes.rs`
- Modify: `src/modules/cadunico/templates.rs`
- Create: `templates/cadunico/error_modal.html`
- Modify: `tests/cadunico_routes.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn post_cadunico_should_return_hx_redirect_when_payload_is_valid() {
    let app = build_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/cadunico")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("hx-request", "true")
                .body(Body::from(
                    "cpf_cnpj=123.456.789-01&fantasia=Cliente+Teste&cep=01001-000&endereco=Rua+A&bairro=Centro&cidade=Sao+Paulo&uf=sp&codigo_ibge=3550308",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["HX-Redirect"], "/cadunico");
}

#[tokio::test]
async fn post_cadunico_should_return_modal_fragment_when_payload_is_invalid() {
    let app = build_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/cadunico")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("hx-request", "true")
                .body(Body::from("fantasia=&cep="))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(html.contains("backend-error-modal"));
    assert!(html.contains("cpf_cnpj is required"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cadunico_routes post_cadunico_should_return_hx_redirect_when_payload_is_valid -- --exact`
Expected: FAIL because the POST route does not exist yet.

**Step 3: Write minimal implementation**

```rust
// src/modules/cadunico/templates.rs
#[derive(Template)]
#[template(path = "cadunico/error_modal.html")]
pub struct CadUnicoErrorModalTemplate<'a> {
    pub title: &'a str,
    pub message: &'a str,
}
```

```rust
// src/modules/cadunico/routes.rs
use axum::extract::Form;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

use super::forms::CadUnicoFormInput;
use super::service::CadUnicoService;
use super::templates::CadUnicoErrorModalTemplate;

pub async fn submit(Form(input): Form<CadUnicoFormInput>) -> Response {
    match CadUnicoService::validate(input) {
        Ok(_) => {
            let mut headers = HeaderMap::new();
            headers.insert("HX-Redirect", HeaderValue::from_static("/cadunico"));
            (StatusCode::OK, headers).into_response()
        }
        Err(error) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            CadUnicoErrorModalTemplate {
                title: "Erro ao salvar",
                message: &error.to_string(),
            }
            .render()
            .unwrap(),
        )
            .into_response(),
    }
}
```

```html
<!-- templates/cadunico/error_modal.html -->
<div id="backend-error-modal" class="modal" role="dialog" aria-modal="true" aria-labelledby="backend-error-title" tabindex="-1">
  <div class="modal__card">
    <h2 id="backend-error-title">{{ title }}</h2>
    <p>{{ message }}</p>
    <button type="button" data-close-modal>Fechar</button>
  </div>
</div>
```

```rust
// src/app.rs
pub fn build_app() -> Router {
    Router::new()
        .route("/", get(routes::home))
        .route("/cadunico", get(routes::index).post(routes::submit))
        .route("/cadunico/criar", get(routes::create))
        .nest_service("/assets", ServeDir::new("assets"))
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test cadunico_routes post_cadunico_should_return_hx_redirect_when_payload_is_valid -- --exact`
Expected: PASS.

Run: `cargo test --test cadunico_routes post_cadunico_should_return_modal_fragment_when_payload_is_invalid -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/app.rs src/modules/cadunico/routes.rs src/modules/cadunico/templates.rs templates/cadunico/error_modal.html tests/cadunico_routes.rs
git commit -m "feat: handle cadunico submit responses"
```

### Task 6: Implement Browser Controller for Tabs, Keyboard Navigation, Masks, and Modal Focus

**Files:**
- Modify: `package.json`
- Create: `assets/scripts/cadunico-form.js`
- Create: `assets/scripts/cadunico-form.test.js`
- Modify: `templates/cadunico/create.html`
- Modify: `assets/styles/app.css`

**Step 1: Write the failing test**

```js
// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import {
  activateNextField,
  activatePreviousField,
  nextTabId,
  normalizeCpfCnpj,
  normalizeCep,
} from "./cadunico-form.js";

describe("cadunico keyboard helpers", () => {
  it("activateNextField should move to the next focusable field", () => {
    const fields = [{ id: "a" }, { id: "b" }, { id: "c" }];
    expect(activateNextField(fields, "a")).toBe("b");
  });

  it("activatePreviousField should move to the previous focusable field", () => {
    const fields = [{ id: "a" }, { id: "b" }, { id: "c" }];
    expect(activatePreviousField(fields, "c")).toBe("b");
  });

  it("nextTabId should move left and right through ordered tabs", () => {
    const tabs = ["dados-principais", "endereco", "parametros", "cobranca"];
    expect(nextTabId(tabs, "endereco", "right")).toBe("parametros");
    expect(nextTabId(tabs, "endereco", "left")).toBe("dados-principais");
  });

  it("normalizeCpfCnpj should strip formatting", () => {
    expect(normalizeCpfCnpj("123.456.789-01")).toBe("12345678901");
  });

  it("normalizeCep should strip formatting", () => {
    expect(normalizeCep("01001-000")).toBe("01001000");
  });
});
```

**Step 2: Run test to verify it fails**

Run: `bunx vitest run assets/scripts/cadunico-form.test.js`
Expected: FAIL because the script module does not exist yet and `jsdom` is not installed.

**Step 3: Write minimal implementation**

```json
{
  "devDependencies": {
    "@types/bun": "latest",
    "jsdom": "^26.0.0",
    "oxfmt": "^0.36.0",
    "oxlint": "^1.51.0",
    "vitest": "^4.0.18"
  }
}
```

```js
// assets/scripts/cadunico-form.js
export function digitsOnly(value) {
  return value.replace(/\D+/g, "");
}

export function normalizeCpfCnpj(value) {
  return digitsOnly(value).slice(0, 14);
}

export function normalizeCep(value) {
  return digitsOnly(value).slice(0, 8);
}

export function activateNextField(fields, currentId) {
  const index = fields.findIndex((field) => field.id === currentId);
  return index >= 0 && index < fields.length - 1 ? fields[index + 1].id : currentId;
}

export function activatePreviousField(fields, currentId) {
  const index = fields.findIndex((field) => field.id === currentId);
  return index > 0 ? fields[index - 1].id : currentId;
}

export function nextTabId(tabs, currentId, direction) {
  const index = tabs.indexOf(currentId);
  if (index === -1) return currentId;
  if (direction === "right") return tabs[Math.min(index + 1, tabs.length - 1)];
  return tabs[Math.max(index - 1, 0)];
}

export function bootstrapCadUnicoForm(root = document.querySelector("[data-cadunico-root]")) {
  if (!root) return;

  // Wire Tab/Shift+Tab, ArrowUp/ArrowDown, Ctrl+ArrowLeft, Ctrl+ArrowRight, Ctrl+S, and Esc here.
  // Keep the event handling in this file only.
}
```

Then run: `bun install`

```html
<!-- templates/cadunico/create.html -->
{% block body %}
...
<script type="module" src="/assets/scripts/cadunico-form.js"></script>
{% endblock %}
```

Implementation details to keep in this step:

- Maintain a per-tab ordered list of focusable elements using `data-tab-panel`.
- Prevent default browser behavior for `Tab`, `Shift+Tab`, `ArrowUp`, `ArrowDown`, `Ctrl+ArrowLeft`, `Ctrl+ArrowRight`, and `Ctrl+S`.
- Apply masks on `input` events without changing the normalized value contract.
- Listen for HTMX swaps on `#cadunico-modal-root`, focus the modal on error, and restore focus on `Esc`.

**Step 4: Run test to verify it passes**

Run: `bunx vitest run assets/scripts/cadunico-form.test.js`
Expected: PASS.

**Step 5: Commit**

```bash
git add package.json bun.lock assets/scripts/cadunico-form.js assets/scripts/cadunico-form.test.js templates/cadunico/create.html assets/styles/app.css
git commit -m "feat: add cadunico keyboard controller"
```

### Task 7: Lock the HTML Contract for Shortcuts, Modal Root, and Internal-App Styling Hooks

**Files:**
- Modify: `tests/cadunico_routes.rs`
- Modify: `templates/cadunico/create.html`
- Modify: `assets/styles/app.css`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn get_cadunico_create_should_expose_shortcuts_and_modal_hooks() {
    let app = build_app();

    let response = app
        .oneshot(Request::builder().uri("/cadunico/criar").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("Ctrl+S envia"));
    assert!(html.contains("id=\"cadunico-modal-root\""));
    assert!(html.contains("data-cadunico-root"));
    assert!(html.contains("class=\"shell shell--wide\""));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cadunico_routes get_cadunico_create_should_expose_shortcuts_and_modal_hooks -- --exact`
Expected: FAIL if any of the help text or hook attributes drifted during implementation.

**Step 3: Write minimal implementation**

```html
<!-- templates/cadunico/create.html -->
<main class="shell shell--wide" data-cadunico-root>
  <header class="page-header">
    <div>
      <p class="eyebrow">Cadastro Unico</p>
      <h1>Novo cadastro</h1>
    </div>
    <p class="shortcut-help">
      Tab/Shift+Tab ou setas verticais navegam campos. Ctrl + setas horizontais trocam tabs. Ctrl+S envia.
    </p>
  </header>
  ...
  <div id="cadunico-modal-root" aria-live="assertive"></div>
</main>
```

```css
/* assets/styles/app.css */
body {
  margin: 0;
  min-height: 100vh;
  background: radial-gradient(circle at top left, #fffaf3 0%, var(--bg) 42%, #efe6da 100%);
  color: var(--ink);
  font: 400 16px/1.4 "IBM Plex Sans", "Segoe UI", sans-serif;
}

.shell--wide {
  min-height: 100vh;
  display: grid;
  grid-template-rows: auto 1fr;
  padding: 24px;
}

.tab-panel {
  max-height: calc(100vh - 240px);
  overflow: auto;
}

:focus-visible {
  outline: 3px solid var(--focus);
  outline-offset: 2px;
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test cadunico_routes get_cadunico_create_should_expose_shortcuts_and_modal_hooks -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add tests/cadunico_routes.rs templates/cadunico/create.html assets/styles/app.css
git commit -m "feat: finalize cadunico page contract"
```

## Full Verification Checklist

Run these before calling the slice complete:

1. `cargo test`
   Expected: all Rust unit and integration tests pass.
2. `bunx vitest run assets/scripts/cadunico-form.test.js`
   Expected: keyboard helper and mask tests pass.
3. `cargo clippy --all-targets --all-features --locked -- -D warnings`
   Expected: no warnings.
4. Manual browser verification:
   - Open `/cadunico/criar`
   - Confirm the page fits inside the viewport without an endless page form
   - Confirm `Tab`, `Shift+Tab`, `ArrowUp`, and `ArrowDown` move within the current tab
   - Confirm `Ctrl+ArrowLeft` and `Ctrl+ArrowRight` switch tabs
   - Confirm `Ctrl+S` submits the form
   - Confirm invalid submit opens the modal
   - Confirm `Esc` closes the modal and restores focus
   - Confirm valid submit lands on `/cadunico`

## Notes for the Implementer

- Keep backend errors typed with `thiserror`; do not use `unwrap()` or `expect()` outside tests.
- Prefer borrowed helpers and small pure functions for normalization instead of cloning large temporary values.
- Keep the browser controller in one file so keyboard behavior stays auditable.
- Do not add SQLX queries in this slice; persistence is explicitly next-step work.
