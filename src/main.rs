use anyhow::Result;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, erp_koten::app::build_app()).await?;

    Ok(())
}
