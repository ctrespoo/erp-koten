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
