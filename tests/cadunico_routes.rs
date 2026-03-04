use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use erp_koten::app::build_app;
use tower::ServiceExt;

#[tokio::test]
async fn get_cadunico_index_should_return_ok() {
    let app = build_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/cadunico")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn get_cadunico_create_should_return_ok() {
    let app = build_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/cadunico/criar")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn get_cadunico_index_should_render_html_layout() {
    let app = build_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/cadunico")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("Cadastro Unico"));
    assert!(html.contains("/assets/styles/app.css"));
    assert!(html.contains("/cadunico/criar"));
}

#[tokio::test]
async fn get_cadunico_create_should_render_tabbed_form_and_all_schema_fields() {
    let app = build_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/cadunico/criar")
                .body(Body::empty())
                .unwrap(),
        )
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

#[tokio::test]
async fn get_cadunico_create_should_render_visible_labels_for_primary_fields() {
    let app = build_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/cadunico/criar")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("CPF / CNPJ"));
    assert!(html.contains("Fantasia"));
    assert!(html.contains("CEP"));
    assert!(html.contains("Email"));
    assert!(html.contains("Novo cadastro"));
}

#[tokio::test]
async fn get_cadunico_create_should_expose_shortcuts_and_modal_hooks() {
    let app = build_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/cadunico/criar")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains(
        "Tab/Shift+Tab ou setas verticais navegam campos. Ctrl + setas horizontais trocam tabs. Ctrl+S envia."
    ));
    assert!(html.contains("class=\"theme-dark\""));
    assert!(html.contains("id=\"cadunico-modal-root\""));
    assert!(html.contains("data-cadunico-root"));
    assert!(html.contains("class=\"shell shell--wide\""));
}

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
async fn post_cadunico_should_accept_phone_when_sent_as_single_form_value() {
    let app = build_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/cadunico")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("hx-request", "true")
                .body(Body::from(
                    "cpf_cnpj=123.456.789-01&fantasia=Cliente+Teste&telefones=%2811%29+99999-9999&cep=01001-000&endereco=Rua+A&bairro=Centro&cidade=Sao+Paulo&uf=sp&codigo_ibge=3550308",
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
                .body(Body::from("telefones=&fantasia=&cep="))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("backend-error-modal"));
    assert!(html.contains("Revise os campos destacados e tente novamente."));
    assert!(html.contains("data-invalid-fields=\"cpf_cnpj,fantasia,cep,endereco,bairro,cidade,uf,codigo_ibge\""));
}
