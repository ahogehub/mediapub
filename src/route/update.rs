use std::io;

use actix_web::{HttpResponse, Responder, web};
use deadpool_postgres::Pool;
use mongodb::Client;

pub async fn update(psql_pool:web::Data<Pool>,monogo_pool:web::Data<Client>)->io::Result<impl Responder>{
    
    Ok(HttpResponse::Ok().finish())
}