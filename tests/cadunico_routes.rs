use axum::body::{to_bytes, Body};
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
        .oneshot(Request::builder().uri("/cadunico").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("Cadastro Unico"));
    assert!(html.contains("/assets/styles/app.css"));
    assert!(html.contains("/cadunico/criar"));
}
