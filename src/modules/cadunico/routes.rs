use askama::Template;
use axum::{
    extract::Form,
    http::{HeaderMap, HeaderValue},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

use super::{
    forms::CadUnicoFormInput,
    service::CadUnicoService,
    templates::{CadUnicoCreateTemplate, CadUnicoErrorModalTemplate, CadUnicoIndexTemplate, TABS},
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

pub async fn index() -> Response {
    match render_html(&CadUnicoIndexTemplate) {
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

pub async fn submit(Form(input): Form<CadUnicoFormInput>) -> Response {
    match CadUnicoService::validate(input) {
        Ok(_) => {
            let mut headers = HeaderMap::new();
            headers.insert("HX-Redirect", HeaderValue::from_static("/cadunico"));
            (StatusCode::OK, headers).into_response()
        }
        Err(error) => {
            let message = error.to_string();
            let template = CadUnicoErrorModalTemplate {
                title: "Erro ao salvar",
                message: &message,
            };

            match template.render() {
                Ok(html) => (StatusCode::UNPROCESSABLE_ENTITY, html).into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
    }
}
