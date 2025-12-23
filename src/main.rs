mod db;
mod web;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv::dotenv();
    tracing_subscriber::fmt::init();
    let db_pool = db::connect().await?;

    sqlx::migrate!().run(&db_pool).await?;

    tokio::spawn(async move {
        if let Err(e) = web::run(db_pool).await {
            tracing::error!("{e}");
        }
    });

    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down");
    Ok(())
}
