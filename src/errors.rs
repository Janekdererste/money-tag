use askama::Template;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::error::Error;

pub struct AppError(Box<dyn Error>);

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate<'a> {
    msg: &'a str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let msg = format!("{}", self.0);
        let template = ErrorTemplate { msg: msg.as_str() };
        (StatusCode::INTERNAL_SERVER_ERROR, template).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<mongodb::error::Error>,
{
    fn from(value: E) -> Self {
        AppError(Box::new(value.into()))
    }
}
