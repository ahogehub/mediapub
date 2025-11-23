use std::{fmt::format, fs::File, io::{self, Write}};

use actix_cors::Cors;
use actix_multipart::form::{MultipartForm, MultipartFormConfig, tempfile::TempFile};
use actix_web::{App, HttpResponse, HttpServer, Responder, get, http::{StatusCode, header::{CROSS_ORIGIN_RESOURCE_POLICY, ContentType}}, web};
use uuid::Uuid;

const MAX_PAYLOAD_SIZE:usize = 1024 * 1024 * 1024;

#[actix_web::main]
async fn  main() -> io::Result<()>{
    HttpServer::new(move ||{
        App::new()
        .app_data(web::PayloadConfig::new(MAX_PAYLOAD_SIZE))
        .app_data(MultipartFormConfig::default().total_limit(MAX_PAYLOAD_SIZE).memory_limit(MAX_PAYLOAD_SIZE))
        .wrap(
            Cors::default()
            .allow_any_origin()
            .allow_any_method()
        )
        .service(ping)
        .service(
            web::resource("/upload")
            .route(web::get().to(index))
            .route(web::post().to(upload))
        )
    })
    .bind(("127.0.0.1",8080))?
    .workers(2)
    .run()
    .await
}

#[get("/ping")]
async fn ping() -> io::Result<impl Responder>{
    Ok(HttpResponse::build(StatusCode::OK).content_type(ContentType::plaintext()).body("hello world!"))
}

#[derive(Debug,MultipartForm)]
struct UploadForm{
    #[multipart(limit= "500MB")]
    file:Vec<TempFile>
}

const DESTINATION:&str = "./tmp";
async fn upload(MultipartForm(form):MultipartForm<UploadForm>) -> io::Result<impl Responder> {
    for f in form.file.into_iter(){
        match f.content_type {
            Some(ct_type)=>{
                println!("Content_Type is {}",ct_type.essence_str())
            },
            None=>{
                println!("Content_Type is none")
            }
        }
        println!("Size:{}",f.size);
        // take ownership of the filename once so we don't move it multiple times or borrow a temporary
        let filename = match f.file_name {
            Some(name) => name,
            None => {
                println!("Name: none");
                return Ok(HttpResponse::BadRequest().body("extention was not found."));
            }
        };
        println!("Name:{}", filename);
        let ext = filename.rsplit('.').next();
        if ext == None {return Ok(HttpResponse::BadRequest().body("extention was not found."))}
        let ext = ext.unwrap();
        let new_filename = format!("{}.{}",Uuid::new_v4(),ext);
        let path = format!("{}/{}",DESTINATION,new_filename);
        match f.file.persist(&path) {
            Ok(_)=>println!("{} saved successfully",filename),
            Err(_)=>println!("{} failed to save",filename)
        };
    };
    let html = r#"
    <html>
        <head>
            <title?>Thx for uploading!</title?
        </head>
        <body>
            <h1>THX</h1>
        </body>
    </html>
    "#;
    Ok(HttpResponse::build(StatusCode::OK).content_type(ContentType::html()).body(html))
}

async fn index()-> io::Result<impl Responder>{
    let html = r#"
    <html>
        <head>
            <title>uploader</title>
        </head>
        <body>
            <form action="/upload" method="post" enctype="multipart/form-data">
                <input type="file" name="file"/>
                <button type="submit">Submit</button>
            </form>
        </body>
       
        
    </html>
    "#;
    Ok(HttpResponse::build(StatusCode::OK).content_type(ContentType::html()).body(html))
}