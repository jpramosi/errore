use axum::{
    body::Body,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use errore::*;

use crate::account;

/// Error type that comprises all errors in this crate.
#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Account(#[from] account::Ec),
}

impl Ec {
    fn status_code(&self) -> StatusCode {
        if self.has::<account::Error>() {
            StatusCode::FORBIDDEN
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl IntoResponse for Ec {
    fn into_response(self) -> Response {
        Response::builder()
            .status(self.status_code())
            .header(header::CONTENT_TYPE, "text/plain")
            .body(Body::new(self.to_string()))
            .unwrap()
    }
}
