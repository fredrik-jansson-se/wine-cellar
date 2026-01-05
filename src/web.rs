
mod handlers;
mod markup;
mod error;

use error::*;

pub(crate) const MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024;

struct StateInner {
    db: sqlx::SqlitePool,
}

type State = std::sync::Arc<StateInner>;

type MDResult = std::result::Result<maud::Markup, AppError>;


pub async fn run(db: sqlx::SqlitePool) -> anyhow::Result<()> {
    let state = StateInner { db }.into();
    let router = axum::Router::new()
        .route("/add-wine", axum::routing::post(handlers::add_wine))
        .route("/", axum::routing::get(markup::index))
        .route(
            "/wines/{wine_id}/upload-image",
            axum::routing::get(markup::upload_wine_image),
        )
        .route(
            "/wines/{wine_id}/image",
            axum::routing::get(handlers::wine_image),
        )
        .route(
            "/wines/{wine_id}/edit-image",
            axum::routing::get(markup::image::edit_image),
        )
        .route(
            "/wines/{wine_id}/edit-image",
            axum::routing::post(handlers::edit_image),
        )
        .route(
            "/wines/{wine_id}/image",
            axum::routing::post(handlers::set_wine_image)
                .layer(axum::extract::DefaultBodyLimit::max(MAX_UPLOAD_BYTES)),
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
            "/wines/{wine_id}/consume",
            axum::routing::get(markup::consume_wine),
        )
        .route(
            "/wines/{wine_id}/consume",
            axum::routing::post(handlers::consume_wine),
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
        .route("/wine-table-body", axum::routing::get(markup::wine_table_body))
        .with_state(state);

    let lap = std::env::var("WINE_LAP").unwrap_or("0.0.0.0:20000".to_owned());
    tracing::info!("Listening: {lap}");
    let listener = tokio::net::TcpListener::bind(lap).await?;
    tracing::info!("Starting web server");
    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}
