use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use actix_web::ResponseError;
use errore::*;

use crate::account;

/// Error type that comprises all errors in this crate.
#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Account(#[from] account::Ec),
}

impl ResponseError for Ec {
    fn status_code(&self) -> StatusCode {
        if self.has::<account::Error>() {
            StatusCode::FORBIDDEN
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code())
            .content_type(ContentType::plaintext())
            .body(self.to_string())
    }
}
