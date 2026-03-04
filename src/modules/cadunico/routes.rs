use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

use super::templates::CadUnicoIndexTemplate;

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

pub async fn create() -> Html<&'static str> {
    Html("<h1>Novo Cadastro Unico</h1>")
}
