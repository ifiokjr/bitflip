pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
#[cfg(feature = "ssr")]
impl axum::response::IntoResponse for AppError {
	fn into_response(self) -> axum::response::Response {
		(
			http::StatusCode::INTERNAL_SERVER_ERROR,
			format!("App error: {}", self.0),
		)
			.into_response()
	}
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to
// turn them into `Result<_, AppError>`. That way you don't need to do that
// manually.
impl<E> From<E> for AppError
where
	E: Into<anyhow::Error>,
{
	fn from(err: E) -> Self {
		Self(err.into())
	}
}
