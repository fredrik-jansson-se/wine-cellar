mod db;
mod web;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv::dotenv();

    let _otlp_guard = init_tracing_opentelemetry::TracingConfig::production()
        .with_stderr()
        .with_otel(true)
        .init_subscriber()?;

    let db_pool = db::connect().await?;

    tracing::info!("Migrate DB");
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
