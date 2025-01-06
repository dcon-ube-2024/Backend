use actix_web::{HttpResponse, ResponseError};
use anyhow::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AnyhowError {
    #[error(transparent)]
    Anyhow(#[from] Error),
}

impl ResponseError for AnyhowError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().body(self.to_string())
    }
}

pub fn anyhow_error<E: Into<Error>>(error: E) -> AnyhowError {
    AnyhowError::Anyhow(error.into())
}
