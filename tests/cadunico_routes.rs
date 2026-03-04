use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use erp_koten::{
    app::build_app,
    state::{AppState, run_migrations},
};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use std::sync::OnceLock;
use tokio::sync::{Mutex, MutexGuard};
use tower::ServiceExt;

static DB_TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

struct TestDatabase {
    pool: PgPool,
    _guard: MutexGuard<'static, ()>,
}

fn test_app() -> axum::Router {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let pool = PgPoolOptions::new()
        .connect_lazy(&database_url)
        .expect("lazy pool creation should succeed");

    build_app(AppState::new(pool))
}

async fn test_database() -> TestDatabase {
    let mutex = DB_TEST_MUTEX.get_or_init(|| Mutex::new(()));
    let guard = mutex.lock().await;

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let pool = connect_test_database(&database_url).await;

    reset_cadunico_table(&pool).await;

    TestDatabase {
        pool,
        _guard: guard,
    }
}

async fn connect_test_database(database_url: &str) -> PgPool {
    match PgPool::connect(database_url).await {
        Ok(pool) => pool,
        Err(sqlx::Error::Database(error)) if error.code().as_deref() == Some("3D000") => {
            create_test_database(database_url).await;
            PgPool::connect(database_url)
                .await
                .expect("database connection should succeed after database creation")
        }
        Err(error) => panic!("database connection should succeed: {error}"),
    }
}

async fn create_test_database(database_url: &str) {
    let (admin_url, database_name) =
        database_admin_url(database_url).expect("DATABASE_URL should include a database name");
    let admin_pool = PgPool::connect(&admin_url)
        .await
        .expect("admin database connection should succeed");
    let statement = format!("CREATE DATABASE \"{}\"", database_name.replace('"', "\"\""));

    match sqlx::query(&statement).execute(&admin_pool).await {
        Ok(_) => {}
        Err(sqlx::Error::Database(error)) if error.code().as_deref() == Some("42P04") => {}
        Err(error) => panic!("database creation should succeed: {error}"),
    }
}

fn database_admin_url(database_url: &str) -> Option<(String, String)> {
    let (prefix, database_name_with_query) = database_url.rsplit_once('/')?;
    let database_name = database_name_with_query
        .split('?')
        .next()
        .filter(|value| !value.is_empty())?;

    Some((format!("{prefix}/postgres"), database_name.to_string()))
}

async fn reset_cadunico_table(pool: &PgPool) {
    run_migrations(pool)
        .await
        .expect("cadunico migrations should succeed");

    sqlx::query("TRUNCATE TABLE cadunico RESTART IDENTITY")
        .execute(pool)
        .await
        .expect("cadunico cleanup should succeed");
}

async fn insert_cadunico(
    pool: &PgPool,
    cpf_cnpj: &str,
    fantasia: &str,
    cidade: &str,
    uf: &str,
) -> i64 {
    sqlx::query_scalar(
        r#"
        INSERT INTO cadunico (
            cpf_cnpj,
            fantasia,
            cep,
            endereco,
            bairro,
            cidade,
            uf,
            codigo_ibge
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
    )
    .bind(cpf_cnpj)
    .bind(fantasia)
    .bind("01001000")
    .bind("Rua A")
    .bind("Centro")
    .bind(cidade)
    .bind(uf)
    .bind("3550308")
    .fetch_one(pool)
    .await
    .expect("seed insert should succeed")
}

#[tokio::test]
async fn get_cadunico_index_should_return_ok() {
    let database = test_database().await;
    let app = build_app(AppState::new(database.pool.clone()));

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
    let app = test_app();

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
    let database = test_database().await;
    let app = build_app(AppState::new(database.pool.clone()));

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
async fn get_cadunico_index_should_render_keyboard_list_shell() {
    let database = test_database().await;
    let app = build_app(AppState::new(database.pool.clone()));

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

    assert!(html.contains("data-cadunico-list-root"));
    assert!(html.contains("Ctrl+K"));
    assert!(html.contains("Ctrl+N"));
    assert!(html.contains("id=\"cadunico-search\""));
    assert!(html.contains("id=\"cadunico-list-region\""));
}

#[tokio::test]
async fn get_cadunico_index_should_render_existing_records_on_initial_load() {
    let database = test_database().await;
    insert_cadunico(
        &database.pool,
        "10000000001",
        "Registro Inicial",
        "Sao Paulo",
        "SP",
    )
    .await;
    let app = build_app(AppState::new(database.pool.clone()));

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

    assert!(html.contains("Registro Inicial"));
    assert!(!html.contains("Nenhum cadastro encontrado"));
}

#[tokio::test]
async fn get_cadunico_index_should_render_delete_dialog_in_the_shell() {
    let database = test_database().await;
    let app = build_app(AppState::new(database.pool.clone()));

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

    assert!(html.contains("data-delete-dialog"));
    assert!(html.contains("data-row-menu-popover-root"));
}

#[tokio::test]
async fn get_cadunico_list_fragment_should_render_empty_state_when_table_is_empty() {
    let database = test_database().await;
    let app = build_app(AppState::new(database.pool.clone()));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/cadunico/lista")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("Nenhum cadastro encontrado"));
    assert!(html.contains("data-list-empty"));
    assert!(html.contains("data-page-next"));
    assert!(html.contains("data-page-prev"));
}

#[tokio::test]
async fn get_cadunico_list_fragment_should_use_eight_items_as_default_page_size() {
    let database = test_database().await;
    for index in 1..=9 {
        insert_cadunico(
            &database.pool,
            &format!("1000000000{index}"),
            &format!("Registro {index}"),
            "Cidade",
            "SP",
        )
        .await;
    }
    let app = build_app(AppState::new(database.pool.clone()));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/cadunico/lista")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("Registro 9"));
    assert!(html.contains("Registro 2"));
    assert!(!html.contains("Registro 1"));
    assert!(html.contains("data-page-next-cursor=\"2\""));
}

#[tokio::test]
async fn get_cadunico_list_fragment_should_filter_by_query_across_fantasia_and_cidade() {
    let database = test_database().await;
    insert_cadunico(&database.pool, "10000000001", "Mercado Porto Azul", "Curitiba", "PR").await;
    insert_cadunico(&database.pool, "10000000002", "Padaria Aurora", "Porto Alegre", "RS").await;
    insert_cadunico(&database.pool, "10000000003", "Auto Center Horizonte", "Recife", "PE").await;
    let app = build_app(AppState::new(database.pool.clone()));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/cadunico/lista?q=porto&page_size=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("Mercado Porto Azul"));
    assert!(html.contains("Padaria Aurora"));
    assert!(!html.contains("Auto Center Horizonte"));
}

#[tokio::test]
async fn get_cadunico_list_fragment_should_not_render_delete_dialog_markup() {
    let app = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/cadunico/lista")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(!html.contains("data-delete-dialog"));
}

#[tokio::test]
async fn get_cadunico_list_fragment_should_render_next_cursor_when_more_rows_exist() {
    let database = test_database().await;
    insert_cadunico(&database.pool, "10000000001", "Registro 1", "Cidade 1", "SP").await;
    insert_cadunico(&database.pool, "10000000002", "Registro 2", "Cidade 2", "SP").await;
    insert_cadunico(&database.pool, "10000000003", "Registro 3", "Cidade 3", "SP").await;
    insert_cadunico(&database.pool, "10000000004", "Registro 4", "Cidade 4", "SP").await;
    let app = build_app(AppState::new(database.pool.clone()));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/cadunico/lista?page_size=2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("Registro 4"));
    assert!(html.contains("Registro 3"));
    assert!(!html.contains("Registro 2"));
    assert!(html.contains(
        "data-page-next\n      name=\"before\""
    ));
    assert!(html.contains("data-page-next-cursor=\"3\""));
}

#[tokio::test]
async fn get_cadunico_list_fragment_should_render_previous_cursor_when_loading_newer_rows() {
    let database = test_database().await;
    insert_cadunico(&database.pool, "10000000001", "Registro 1", "Cidade 1", "SP").await;
    insert_cadunico(&database.pool, "10000000002", "Registro 2", "Cidade 2", "SP").await;
    insert_cadunico(&database.pool, "10000000003", "Registro 3", "Cidade 3", "SP").await;
    insert_cadunico(&database.pool, "10000000004", "Registro 4", "Cidade 4", "SP").await;
    insert_cadunico(&database.pool, "10000000005", "Registro 5", "Cidade 5", "SP").await;
    let app = build_app(AppState::new(database.pool.clone()));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/cadunico/lista?after=2&page_size=2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains(
        "data-page-prev\n      name=\"after\""
    ));
    assert!(html.contains("Registro 4"));
    assert!(html.contains("Registro 3"));
    assert!(!html.contains("Registro 5"));
    assert!(html.contains("data-page-prev-cursor=\"4\""));
}

#[tokio::test]
async fn delete_cadunico_should_remove_the_selected_record_and_refresh_the_fragment() {
    let database = test_database().await;
    let id = insert_cadunico(&database.pool, "10000000001", "Registro 1", "Cidade 1", "SP").await;
    let app = build_app(AppState::new(database.pool.clone()));

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/cadunico/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("Nenhum cadastro encontrado"));

    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM cadunico")
        .fetch_one(&database.pool)
        .await
        .expect("count query should succeed");
    assert_eq!(remaining, 0);
}

#[tokio::test]
async fn get_cadunico_list_fragment_should_render_edit_and_delete_actions_for_each_row() {
    let database = test_database().await;
    insert_cadunico(&database.pool, "10000000001", "Registro 1", "Cidade 1", "SP").await;
    let app = build_app(AppState::new(database.pool.clone()));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/cadunico/lista")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("data-row-id=\"1\""));
    assert!(html.contains("data-row-name=\"Registro 1\""));
    assert!(html.contains("cadunico-list-actions"));
    assert!(!html.contains("data-row-menu"));
    assert!(!html.contains("data-delete-dialog"));
}

#[tokio::test]
async fn get_cadunico_create_should_render_tabbed_form_and_all_schema_fields() {
    let app = test_app();

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
    let app = test_app();

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
    let app = test_app();

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
    let database = test_database().await;
    let app = build_app(AppState::new(database.pool.clone()));

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

    if response.status() != StatusCode::OK {
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        panic!(
            "unexpected status {status}: {}",
            String::from_utf8_lossy(&body)
        );
    }

    assert_eq!(response.headers()["HX-Redirect"], "/cadunico");
}

#[tokio::test]
async fn post_cadunico_should_accept_phone_when_sent_as_single_form_value() {
    let database = test_database().await;
    let app = build_app(AppState::new(database.pool.clone()));

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

    if response.status() != StatusCode::OK {
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        panic!(
            "unexpected status {status}: {}",
            String::from_utf8_lossy(&body)
        );
    }

    assert_eq!(response.headers()["HX-Redirect"], "/cadunico");
}

#[tokio::test]
async fn post_cadunico_should_return_modal_fragment_when_payload_is_invalid() {
    let app = test_app();

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
    assert!(html.contains(
        "data-invalid-fields=\"cpf_cnpj,fantasia,cep,endereco,bairro,cidade,uf,codigo_ibge\""
    ));
}

#[tokio::test]
async fn post_cadunico_should_persist_full_payload_when_valid() {
    let database = test_database().await;
    let app = build_app(AppState::new(database.pool.clone()));

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/cadunico")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("hx-request", "true")
                .body(Body::from(
                    "cpf_cnpj=12.345.678%2F0001-90&inscricao_estadual=12345&inscricao_municipal=54321&fantasia=Cliente+Completo&inss=9988&crea=5566&email=cliente%40teste.com&telefones=%2811%29+99999-9999&telefones=%2811%29+3333-4444&aniversario=2024-12-31&id_estrangeiro=ABC123&codigo_pais=1058&cep=01001-000&endereco=Rua+A&numero=100&complemento=Sala+2&bairro=Centro&cidade=Sao+Paulo&uf=sp&codigo_ibge=3550308&enviar_nfe=true&enviar_boleto=true&enviar_extrato=true&etiqueta=true&comissao=true&construcao_civil=true&cep_cobranca=20040-020&endereco_cobranca=Rua+B&numero_cobranca=200&complemento_cobranca=Fundos&bairro_cobranca=Comercial&cidade_cobranca=Rio+de+Janeiro&uf_cobranca=rj&codigo_ibge_cobranca=3304557&referencia_cobranca=Ao+lado+da+praca",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    if response.status() != StatusCode::OK {
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        panic!(
            "unexpected status {status}: {}",
            String::from_utf8_lossy(&body)
        );
    }

    assert_eq!(response.headers()["HX-Redirect"], "/cadunico");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM cadunico")
        .fetch_one(&database.pool)
        .await
        .expect("count query should succeed");

    assert_eq!(count, 1);

    let row = sqlx::query(
        r#"
        SELECT cpf_cnpj, fantasia, cep, uf, telefones, aniversario::TEXT AS aniversario, cep_cobranca, uf_cobranca
        FROM cadunico
        "#,
    )
    .fetch_one(&database.pool)
    .await
    .expect("select query should succeed");

    assert_eq!(row.get::<String, _>("cpf_cnpj"), "12345678000190");
    assert_eq!(row.get::<String, _>("fantasia"), "Cliente Completo");
    assert_eq!(row.get::<String, _>("cep"), "01001000");
    assert_eq!(row.get::<String, _>("uf"), "SP");
    assert_eq!(
        row.get::<Vec<String>, _>("telefones"),
        vec!["11999999999".to_string(), "1133334444".to_string()]
    );
    assert_eq!(
        row.get::<Option<String>, _>("aniversario").as_deref(),
        Some("2024-12-31")
    );
    assert_eq!(
        row.get::<Option<String>, _>("cep_cobranca").as_deref(),
        Some("20040020")
    );
    assert_eq!(
        row.get::<Option<String>, _>("uf_cobranca").as_deref(),
        Some("RJ")
    );
}

#[tokio::test]
async fn post_cadunico_should_return_modal_fragment_when_cpf_cnpj_is_duplicated() {
    let database = test_database().await;
    let app = build_app(AppState::new(database.pool.clone()));

    sqlx::query(
        r#"
        INSERT INTO cadunico (
            cpf_cnpj,
            fantasia,
            cep,
            endereco,
            bairro,
            cidade,
            uf,
            codigo_ibge
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind("12345678901")
    .bind("Cliente existente")
    .bind("01001000")
    .bind("Rua A")
    .bind("Centro")
    .bind("Sao Paulo")
    .bind("SP")
    .bind("3550308")
    .execute(&database.pool)
    .await
    .expect("seed insert should succeed");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/cadunico")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("hx-request", "true")
                .body(Body::from(
                    "cpf_cnpj=123.456.789-01&fantasia=Cliente+Duplicado&cep=01001-000&endereco=Rua+A&bairro=Centro&cidade=Sao+Paulo&uf=sp&codigo_ibge=3550308",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("Ja existe um cadastro com este CPF / CNPJ."));
    assert!(html.contains("data-invalid-fields=\"cpf_cnpj\""));
}
