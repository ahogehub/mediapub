use std::io::{self, Error};
use actix_cors::Cors;
use actix_files::NamedFile;
use actix_multipart::form::{MultipartForm, MultipartFormConfig, json::Json, tempfile::TempFile};
use actix_web::{App, HttpResponse, HttpServer, Responder, get, http::{StatusCode, header::ContentType}, web};
use rusqlite::Connection;
use mongodb::Client;
use uuid::Uuid;
use serde::{Deserialize, Serialize};


#[derive(Debug,Serialize,Deserialize,Clone)]
struct Uploader{
    id:Uuid,
    displayname:String
}
#[derive(Debug,Serialize,Deserialize,Clone)]
struct Article{
    id:Uuid,
    title:String,
    creator:String,
    source:String,
    description:String,
    uploader:Uploader
}
#[derive(Serialize)]
struct ResponseJson{ 
    recived_files:Vec<String>
}


const MAX_PAYLOAD_SIZE:usize = 1024 * 1024 * 1024;
const DB_PATH:&str = "./data/database.db";
const MONGODB_URI:&str = "mongodb://localhost:27017";
#[actix_web::main]
async fn  main() -> io::Result<()>{
    //mongodb initialization
    let clinet = Client::with_uri_str(MONGODB_URI).await;
    if clinet.is_err(){
       println!("Error while establish connection to db");
       println!("{}",clinet.as_ref().err().unwrap().to_string());
       return Err(Error::new(io::ErrorKind::Other,clinet.err().unwrap().to_string()))
    };
    let db = clinet.unwrap().database("image");
    let coll = db.collection::<Article>("article");
    let mongo_db_list = db.list_collection_names().await;
    if mongo_db_list.is_err(){
        println!("Error while listing collection names");
        println!("{}",mongo_db_list.as_ref().err().unwrap().to_string());
        return Err(Error::new(io::ErrorKind::Other,mongo_db_list.err().unwrap().to_string()))
    };
    for collection_name in mongo_db_list.unwrap() {
        println!("Collection name: {}", collection_name);
    };
     
    //sqlite database initialization
    let connection = match Connection::open(DB_PATH) {
        Ok(conn) => conn,
        Err(e)=>{
            println!("Error while establish connection to db");
            println!("{}",e.to_string());
            return Err(Error::new(io::ErrorKind::Other,e.to_string()))
            
        }
    };
    match connection.execute(
        "CREATE TABLE IF NOT EXISTS item (
            id TEXT PRIMARY KEY,
            file TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        (),
    ) {
        Ok(_) => println!("table created or already exists"),
        Err(e) => {
            println!("Failed to create table: {}", e);
            return Err(Error::new(io::ErrorKind::Other, e.to_string()))
        }
    };
    println!("===Finish Initialization===");
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
        .service(
            web::resource("/get")
            .route(web::get().to(get))
        )
        .service(
            web::resource("/item/{item_id}")
            .route(web::get().to(get_item))
        )

    })
    .bind(("0.0.0.0",8080))?
    .workers(2)
    .run()
    .await
}

#[get("/ping")]
async fn ping() -> io::Result<impl Responder>{
    Ok(HttpResponse::build(StatusCode::OK).content_type(ContentType::plaintext()).body("hello world!"))
}

async fn get_item(item_id:web::Path<String>)-> io::Result<impl Responder>{
    println!("item_id is {}",item_id);
    Ok(NamedFile::open(format!("{}/{}",DESTINATION,item_id)))
}

async fn get() -> io::Result<impl Responder>{
    let connection = match Connection::open(DB_PATH) {
        Ok(conn)=>conn,
        Err(e)=>{
            println!("Error while establish connection to db");
            println!("{}",e.to_string());
            return Err(Error::new(io::ErrorKind::Other,e.to_string()));
        }
    };
    let mut stmt = match connection.prepare("SELECT file FROM item") {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to prepare statement: {}", e);
            return Err(Error::new(io::ErrorKind::Other, e.to_string()));
        }
    };
    let files_iter = match stmt.query_map([], |row| row.get::<_, String>(0)) {
        Ok(iter) => iter,
        Err(e) => {
            println!("Failed to query: {}", e);
            return Err(Error::new(io::ErrorKind::Other, e.to_string()));
        }
    };
    let mut files:Vec<String> = Vec::new();
    for file in files_iter {
        match file {
            Ok(f) => files.push(f),
            Err(e) => {
                println!("Failed to get file: {}", e);
               return Err(Error::new(io::ErrorKind::Other, e.to_string()));
            }
        }
    }
  
    let body_json = ResponseJson { recived_files:files };
    Ok(HttpResponse::Ok().content_type(ContentType::json()).json(body_json))
}
async fn connect_to_sqlite() -> Result<Connection, Error> {
    match Connection::open(DB_PATH) {
        Ok(conn) => return Ok(conn),
        Err(e) => {
            println!("Failed to connect to SQLite database: {}", e);
            return Err(Error::new(io::ErrorKind::Other, e.to_string()));
        }
    };
}
async fn connect_to_mongo() -> Result<Client, Error> {
    match Client::with_uri_str(MONGODB_URI).await {
        Ok(client) => return Ok(client),
        Err(e) => {
            println!("Failed to connect to MongoDB: {}", e);
            return Err(Error::new(io::ErrorKind::Other, e.to_string()));
        }
    };
}
#[derive(Debug,Deserialize)]
struct UploadJson{
    title:String,
    creator:String,
    source:String,
    description:String,
    uploader:Uploader 
}
#[derive(Debug,MultipartForm)]
struct UploadForm{
    #[multipart(limit= "10MB")]
    file:Vec<TempFile>,
    json:Vec<Json<UploadJson>>
}

const DESTINATION:&str = "./tmp";
async fn upload(MultipartForm(form):MultipartForm<UploadForm>) -> io::Result<impl Responder> {
    if form.file.len() != form.json.len(){
        return Ok(HttpResponse::BadRequest().body("enough json metadata was not provided."));
    };
    println!("request approved");
    //mongo
    let mongo = match connect_to_mongo().await {
        Ok(client) => client,
        Err(_) => {
            return Ok(HttpResponse::ExpectationFailed().finish());
        }
    };
    let coll = mongo.database("image").collection::<Article>("article");
    //sqlite
    let sqlite = match connect_to_sqlite().await {
        Ok(conn) => conn,
        Err(_) => {
            return Ok(HttpResponse::ExpectationFailed().finish());
        }
    };
    println!("database connection established");
    let mut recived_files:Vec<String> = Vec::new();
    for (f,j) in form.file.into_iter().zip(form.json.iter()){
        match f.content_type {
            Some(ct_type)=>{
                println!("Content_Type is {}",ct_type.essence_str())
            },
            None=>{
                println!("Content_Type not found")
            }
        }
        let filename = match f.file_name {
            Some(name) => name,
            None => {
                println!("filename was not found");
                return Ok(HttpResponse::BadRequest().body("filename was not found.")); 
            }
        };
        println!("file check completed");
        println!("Name:{}", filename);
        let ext = filename.rsplit('.').next();
        if ext.is_none() {
            return Ok(HttpResponse::BadRequest().body("extention was not found."));
        }
        let ext = ext.unwrap();
        let uuid = Uuid::new_v4();
        let new_filename = format!("{}.{}",&uuid,ext);
        let path = format!("{}/{}",DESTINATION,new_filename);
        match f.file.persist(&path) {
            Ok(_)=>println!("{} saved successfully",filename),
            Err(_)=>println!("{} failed to save",filename)
        };
        
        let _ = match sqlite.execute("
            INSERT INTO item (id, file)
            VALUES (?1, ?2)
        ",(&uuid.to_string(),&new_filename)
        ) {
            Ok(_) => {},
            Err(e) => {
                println!("Insert Error {}",e.to_string());
                return Ok(HttpResponse::InternalServerError().finish());
            }
        };
        //TODO mongo insert
        let article = Article{
            id:uuid,
            title:j.title.clone(),
            creator:j.creator.clone(),
            source:j.source.clone(),
            description:j.description.clone(),
            uploader:j.uploader.clone()
        };
        let _ = match coll.insert_one(article).await{
            Ok(_) => {
                recived_files.push(new_filename);
            },
            Err(e) => {
                println!("Mongo Insert Error {}",e.to_string());
                return Ok(HttpResponse::InternalServerError().finish());
            } 
        }; 
       
    };
    let response = ResponseJson{
        recived_files:recived_files
    }; 
    Ok(HttpResponse::Ok().content_type(ContentType::json()).json(response))
}
//for debug
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