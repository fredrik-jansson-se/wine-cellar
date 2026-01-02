use axum::response::IntoResponse;

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
        let (err, class) = if self.status.is_client_error() {
            tracing::warn!("{}", self.error);
            (self.error.to_string(), "text-bg-warning")
        } else {
            tracing::error!("{}", self.error);
            ("Internal Error".to_owned(), "text-bg-danger")
        };
        (
            self.status,
            maud::html! {
                div class=(format!("{class} p-3")) {
                    (err)
                }
            },
        )
            .into_response()
    }
}
