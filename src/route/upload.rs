use crate::{
    DESTINATION,
    types::{Post, ResponseFile, UploadFrom},
    utility::{self, generate_response},
};
use actix_multipart::form::MultipartForm;
use actix_web::{
    HttpRequest, HttpResponse, Responder,
    http::header::{AUTHORIZATION, ContentType},
};
use std::io;
use utility::{CredentialType, check_user_validity};
use uuid::Uuid;

pub async fn upload(
    MultipartForm(form): MultipartForm<UploadFrom>,
    request: HttpRequest,
) -> io::Result<impl Responder> {
    if form.file.len() != form.metadata.len() {
        return Ok(HttpResponse::BadRequest().body("metadata count does not match file count."));
    }
    let auth = request.headers().get(AUTHORIZATION);
    if auth.is_none() {
        return Ok(HttpResponse::Unauthorized().body("authorization header not found."));
    }
    let auth_header = match auth.unwrap().to_str() {
        Ok(h) => h,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().body("Invalid authorization header format."));
        }
    };
    let user_id = match check_user_validity(auth_header, CredentialType::SessionToken).await {
        Ok(id)=> id,
        Err(e)=>{
            return Ok(generate_response(&e))
        }
    }; 
    //db connection 
    let mongo = match utility::connect_to_mongo().await {
        Ok(client) => client,
        Err(_) => {
            return Ok(HttpResponse::ExpectationFailed().finish());
        }
    };
    let coll = mongo.database("image").collection::<Post>("article");
    let mut postgres = match utility::connect_to_postgres().await {
        Ok(conn) => conn,
        Err(_) => {
            return Ok(HttpResponse::ExpectationFailed().finish());
        }
    };
    println!("database connection established");
    //db statement
    let statement = match postgres.prepare("INSERT INTO post (post_id, user_id) VALUES ($1, $2)") {
        Ok(stmt) => stmt,
        Err(e) => {
            eprintln!("Failed to prepare PostgreSQL statement: {}", e);
            //TODO will have to delete response text
            return Ok(HttpResponse::InternalServerError().body("Database preparation failed."));
        }
    };
    //file processing
    let mut received_files: Vec<String> = Vec::new();
    for (file, metadata) in form.file.into_iter().zip(form.metadata.0.into_iter()) {
        //validate content type
        match &file.content_type {
            Some(ct_type) => {
                println!("Content-Type: {}", ct_type.essence_str());
            }
            None => {
                return Ok(HttpResponse::BadRequest().body("Content-Type header missing."));
            }
        }
        //extract filename
        let filename = match &file.file_name {
            Some(name) => name.clone(),
            None => {
                eprintln!("filename was not found");
                return Ok(HttpResponse::BadRequest().body("filename was not found."));
            }
        };
        //extract and validate extension
        let ext = match filename.rsplit('.').next() {
            Some(e) => e,
            None => {
                return Ok(HttpResponse::BadRequest().body("extension was not found."));
            }
        };
        //generate new filename
        let post_id = Uuid::new_v4();
        let new_filename = format!("{}.{}", &post_id, ext);
        let path = format!("{}/{}", DESTINATION, new_filename);
        //save file to disk
        match file.file.persist(&path) {
            Ok(_) => println!("{} saved successfully", filename),
            Err(e) => {
                eprintln!("{} failed to save: {}", filename, e);
                return Ok(
                    HttpResponse::InternalServerError().body("Failed to save uploaded file.")
                );
            }
        }
        //inset into postgres
        if let Err(e) = postgres.execute(
            &statement,
            &[&post_id.as_bytes().to_vec(), &user_id.as_bytes().to_vec()],
        ) {
            eprintln!("PostgreSQL Insert Error: {}", e);
            return Ok(HttpResponse::InternalServerError().body("Failed to store post metadata in database."));
        }
        //insert into mongodb
        let article = Post {
            post_id,
            title: metadata.title.clone(),
            creator: metadata.creator.clone(),
            source: metadata.source.clone(),
            description: metadata.description.clone(),
            uploader: user_id,
        };
        match coll.insert_one(article).await {
            Ok(_) => {
                println!("Inserted post {} to MongoDB", post_id);
                received_files.push(new_filename);
            }
            Err(e) => {
                eprintln!("MongoDB Insert Error: {}", e);
                return Ok(HttpResponse::InternalServerError().body("Failed to store post in MongoDB."));
            }
        }
    }

    let response = ResponseFile{
        file : received_files,
    };

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .json(response))
}


