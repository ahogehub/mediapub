use actix_cors::Cors;
use actix_multipart::form::MultipartFormConfig;
use actix_web::{
    App, HttpResponse, HttpServer, Responder,
    http::{StatusCode, header::ContentType},
    web::{self},
};
use std::io::{self, Error};
use mediapub::{
    route::upload::upload,
    route::items::{get_all, get_one},
    route::ping::ping,
    init,
    MAX_PAYLOAD_SIZE,
};
#[actix_web::main]
async fn main() -> Result<(), Error> {
    match init::database().await {
        Ok(_) => {}
        Err(e) => {
            println!("Database initialization failed: {}", e);
            return Err(Error::new(
                io::ErrorKind::Other,
                "Database initialization failed",
            ));
        }
    }
    HttpServer::new(move || {
        App::new()
            .app_data(web::PayloadConfig::new(MAX_PAYLOAD_SIZE))
            .app_data(
                MultipartFormConfig::default()
                    .total_limit(MAX_PAYLOAD_SIZE)
                    .memory_limit(MAX_PAYLOAD_SIZE),
            )
            .wrap(Cors::default().allow_any_origin().allow_any_method())
            .service(
                web::resource("/ping")
                    .route(web::get().to(ping)))
            .service(
                web::resource("/upload")
                    .route(web::get().to(index))
                    .route(web::post().to(upload)),
            )
            .service(
                web::resource("/item")
                    .route(web::get().to(get_all)))
            .service(
                web::resource("/item/{item_id}")
                    .route(web::get().to(get_one)))
    })
    .bind(("0.0.0.0", 8080))?
    .workers(2)
    .run()
    .await
}

//for debug
async fn index() -> io::Result<impl Responder> {
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
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::html())
        .body(html))
}
