use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PrerenderError {
    #[error("Invalid Url.")]
    InvalidUrl,

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}

impl ResponseError for PrerenderError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> HttpResponse {
        let res = HttpResponse::with_body(self.status_code(), self.to_string());
        res.map_into_boxed_body()
    }
}
