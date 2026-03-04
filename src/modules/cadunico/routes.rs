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
