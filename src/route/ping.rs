use std::io;
use actix_web::{Responder, HttpResponse,http::StatusCode, http::header::ContentType};
pub async fn ping() -> io::Result<impl Responder>{
    Ok(HttpResponse::build(StatusCode::OK).content_type(ContentType::plaintext()).body("sex on the beach!"))
}