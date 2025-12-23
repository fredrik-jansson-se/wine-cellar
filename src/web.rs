mod handlers;
mod markup;

struct StateInner {
    db: sqlx::SqlitePool,
}

type State = std::sync::Arc<tokio::sync::Mutex<StateInner>>;

pub async fn run(db: sqlx::SqlitePool) -> anyhow::Result<()> {
    let state = std::sync::Arc::new(tokio::sync::Mutex::new(StateInner { db }));
    let router = axum::Router::new()
        .route("/add-wine", axum::routing::get(markup::add_wine))
        .route("/add-wine", axum::routing::post(handlers::add_wine_post))
        .route("/", axum::routing::get(markup::index))
        .route(
            "/wines/{wine_id}/image",
            axum::routing::get(markup::upload_wine_image),
        )
        .route(
            "/wines/{wine_id}/image",
            axum::routing::post(handlers::set_wine_image)
                // Disable body limit on file
                .layer(axum::extract::DefaultBodyLimit::disable()),
        )
        .route(
            "/wines/{wine_id}/comment",
            axum::routing::get(markup::add_comment),
        )
        .route(
            "/wines/{wine_id}/comment",
            axum::routing::post(handlers::add_comment),
        )
        .route(
            "/wines/{wine_id}/drink",
            axum::routing::get(markup::drink_wine),
        )
        .route(
            "/wines/{wine_id}/drink",
            axum::routing::post(handlers::drink_wine),
        )
        .route("/wines/{wine_id}/buy", axum::routing::get(markup::buy_wine))
        .route(
            "/wines/{wine_id}/buy",
            axum::routing::post(handlers::buy_wine),
        )
        .route(
            "/wines/{wine_id}/grapes",
            axum::routing::get(markup::edit_wine_grapes),
        )
        .route(
            "/wines/{wine_id}/grapes",
            axum::routing::post(handlers::post_wine_grapes),
        )
        .route(
            "/wines/{wine_id}",
            axum::routing::get(markup::wine_information),
        )
        .route(
            "/wines/{wine_id}",
            axum::routing::delete(handlers::delete_wine),
        )
        .route("/wines", axum::routing::get(markup::wine_table))
        .with_state(state);

    let lap = std::env::var("WINE_LAP").unwrap_or("0.0.0.0:20000".to_owned());
    let listener = tokio::net::TcpListener::bind(lap).await?;
    tracing::info!("Starting web server");
    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}
