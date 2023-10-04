use axum::response::IntoResponse;
use hyper::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("error reading templates: {0}")]
    TemplatesError(#[from] tera::Error),
    #[error("error starting web server: {0}")]
    WebServerError(#[from] hyper::Error),
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("error rendering template: {0}")]
    TemplateError(#[from] tera::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            Self::TemplateError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        return (status, self.to_string()).into_response();
    }
}
