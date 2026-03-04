use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    http::{HeaderMap, HeaderValue},
    response::{Html, IntoResponse, Response},
};

use crate::state::AppState;

use super::{
    errors::CadUnicoFormError,
    forms::CadUnicoFormInput,
    service::{CadUnicoListInput, CadUnicoService, CadUnicoServiceError},
    templates::{
        CadUnicoCreateTemplate, CadUnicoErrorModalTemplate, CadUnicoIndexTemplate,
        CadUnicoListItemView, CadUnicoListPageView, CadUnicoListPartialTemplate, TABS,
    },
};

fn render_html<T>(template: &T) -> Result<Html<String>, StatusCode>
where
    T: Template,
{
    template
        .render()
        .map(Html)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn home() -> Html<&'static str> {
    Html("<h1>ERP Koten</h1>")
}

pub async fn index(
    State(state): State<AppState>,
    Query(query): Query<CadUnicoListInput>,
) -> Response {
    let search_value = query.search.clone().unwrap_or_default();
    let page = match CadUnicoService::list(&state.db, query).await {
        Ok(page) => map_list_page(page, &search_value),
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

pub async fn create() -> Response {
    match render_html(&CadUnicoCreateTemplate { tabs: TABS }) {
        Ok(html) => html.into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn list_fragment(
    State(state): State<AppState>,
    Query(query): Query<CadUnicoListInput>,
) -> Response {
    let search_value = query.search.clone().unwrap_or_default();
    let page = match CadUnicoService::list(&state.db, query).await {
        Ok(page) => map_list_page(page, &search_value),
        Err(CadUnicoServiceError::Unexpected(_)) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        Err(CadUnicoServiceError::Form(_)) => {
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    match render_html(&CadUnicoListPartialTemplate { page: &page }) {
        Ok(html) => html.into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn destroy(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<CadUnicoListInput>,
) -> Response {
    let search_value = query.search.clone().unwrap_or_default();

    if let Err(error) = CadUnicoService::delete(&state.db, id).await {
        return match error {
            CadUnicoServiceError::Form(_) => StatusCode::BAD_REQUEST.into_response(),
            CadUnicoServiceError::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
    }

    let page = match CadUnicoService::list(&state.db, query).await {
        Ok(page) => map_list_page(page, &search_value),
        Err(CadUnicoServiceError::Unexpected(_)) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        Err(CadUnicoServiceError::Form(_)) => {
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    match render_html(&CadUnicoListPartialTemplate { page: &page }) {
        Ok(html) => html.into_response(),
        Err(status) => status.into_response(),
    }
}

fn render_error_modal(error: CadUnicoFormError) -> Response {
    let message = error.to_string();
    let invalid_fields = error.invalid_fields_csv();
    let template = CadUnicoErrorModalTemplate {
        title: "Erro ao salvar",
        message: &message,
        invalid_fields: &invalid_fields,
    };

    match template.render() {
        Ok(html) => (StatusCode::UNPROCESSABLE_ENTITY, html).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn submit(State(state): State<AppState>, body: String) -> Response {
    let input = CadUnicoFormInput::from_form_body(&body);

    match CadUnicoService::create(&state.db, input).await {
        Ok(()) => {
            let mut headers = HeaderMap::new();
            headers.insert("HX-Redirect", HeaderValue::from_static("/cadunico"));
            (StatusCode::OK, headers).into_response()
        }
        Err(CadUnicoServiceError::Form(error)) => render_error_modal(error),
        Err(CadUnicoServiceError::Unexpected(_)) => {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn map_list_page(
    page: super::repository::CadUnicoListPage,
    search_value: &str,
) -> CadUnicoListPageView {
    CadUnicoListPageView {
        heading: "Cadastros",
        search_value: search_value.to_owned(),
        items: page
            .items
            .into_iter()
            .map(|item| CadUnicoListItemView {
                id: item.id,
                cpf_cnpj: item.cpf_cnpj,
                fantasia: item.fantasia,
                cidade: item.cidade,
                uf: item.uf,
            })
            .collect(),
        next_cursor: page.next_cursor,
        prev_cursor: page.prev_cursor,
    }
}
