use actix_web::HttpResponse;
use actix_web::{Responder,web::Path};
use actix_files::NamedFile;
use std::io;
use crate::DESTINATION;
use crate::types::ResponseFile;
use crate::utility;
pub async fn get_one(item_id:Path<String>)-> io::Result<impl Responder>{
    println!("item_id is {}",item_id);
    Ok(NamedFile::open(format!("{}/{}",DESTINATION,item_id)))
}

pub async fn get_all() -> io::Result<impl Responder>{
    let mut postgres = match utility::connect_to_postgres().await {
        Ok(conn) => conn,
        Err(_) => {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to connect to Postgres"));
        } 
    };
    let result = postgres.query("SELECT post_id FROM post", &[]);
    let ids = match result {
        Ok(row_vec)=>{row_vec.iter().map(|file|->String{file.get::<_,String>(0)}).collect::<Vec<String>>()},
        Err(e)=>{
            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Query failed: {}", e)));
        }
    };
    let response = ResponseFile{
        file:ids
    };
    Ok(HttpResponse::Ok().json(response))
}