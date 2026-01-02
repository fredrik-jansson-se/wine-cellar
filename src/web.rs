use axum::response::IntoResponse;

mod handlers;
mod markup;

pub(crate) const MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024;

struct StateInner {
    db: sqlx::SqlitePool,
}

type State = std::sync::Arc<StateInner>;

type MDResult = std::result::Result<maud::Markup, AppError>;

pub(crate) struct AppError {
    error: anyhow::Error,
    status: axum::http::StatusCode,
}

impl AppError {
    pub(crate) fn bad_request<E>(err: E) -> Self
    where
        E: Into<anyhow::Error>,
    {
        Self {
            error: err.into(),
            status: axum::http::StatusCode::BAD_REQUEST,
        }
    }

    pub(crate) fn payload_too_large<E>(err: E) -> Self
    where
        E: Into<anyhow::Error>,
    {
        Self {
            error: err.into(),
            status: axum::http::StatusCode::PAYLOAD_TOO_LARGE,
        }
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self {
            error: err.into(),
            status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{}", self.error);
        let class = if self.status.is_client_error() {
            "text-bg-warning"
        } else {
            "text-bg-danger"
        };
        (
            self.status,
            maud::html! {
                div class=(format!("{class} p-3")) {
                    (self.error)
                }
            },
        )
            .into_response()
    }
}

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
        .with_state(state);

    let lap = std::env::var("WINE_LAP").unwrap_or("0.0.0.0:20000".to_owned());
    tracing::info!("Listening: {lap}");
    let listener = tokio::net::TcpListener::bind(lap).await?;
    tracing::info!("Starting web server");
    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}
