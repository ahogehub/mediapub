use actix_web::{HttpRequest, HttpResponse, Responder};
use postgres::NoTls;
use serde::Deserialize;
use uuid::Uuid;
use crate::{MONGODB_URI, POSTGRES_URI};

//db util
pub async fn connect_to_postgres() -> std::io::Result<postgres::Client>{
    match postgres::Client::connect(POSTGRES_URI, NoTls) {
        Ok(client) => return Ok(client),
        Err(e) => {
            println!("Failed to connect to Postgres database: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
        }
    }
}

pub async fn connect_to_mongo() -> std::io::Result<mongodb::Client>{
    match mongodb::Client::with_uri_str(MONGODB_URI).await {
        Ok(client) => return Ok(client),
        Err(e) => {
            println!("Failed to connect to MongoDB: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
        }
    }
}
#[derive(Debug,Deserialize)]
pub enum CredentialType {
    SessionToken,
    DevToken,
}
use crate::errors::{
    AHError::{
      InvalidCredential,
     UserInactive,
      AccountSuspended  
    },
    DBError::{
        ConnectionFailed,
        QueryFailed},
    DBType::{
        Mongodb,
        Postgres
    },
    ErrorKind::{
        self,
        DatabaseError,
        AuthError
    }
};
fn extract_user_id_from_row(row: &postgres::Row) -> Result<Uuid, ErrorKind> {
    let user_id_bytes: Vec<u8> = row.get(0);
    match Uuid::from_slice(&user_id_bytes) {
        Ok(uuid) => Ok(uuid),
        Err(e) => {
            println!("Invalid user_id format: {}", e);
            Err(DatabaseError(QueryFailed(Postgres)))
        }
    }
}
pub async fn check_user_validity(credential:&str,credential_type:CredentialType) -> Result<Uuid,ErrorKind>{
    let mut postgres = match connect_to_postgres().await {
        Ok(client) => client,
        Err(_) => {
            return Err(DatabaseError(ConnectionFailed(Postgres))) 
        }
    };
    let user_id = match credential_type {
        CredentialType::DevToken => {
            match postgres.query_one(
                    "SELECT user_id FROM dev_token WHERE token_hash = $1 AND is_revoked = false",
                    &[&credential]
                ){
                Ok(row) => {
                    extract_user_id_from_row(&row) 
                },
                Err(_) => {
                    Err(DatabaseError(QueryFailed(Postgres)))
                }}
        },
        CredentialType::SessionToken => {
            match postgres.query_one(
                    "SELECT user_id FROM session WHERE session_token = $1 AND is_revoked = false",
                    &[&credential]
                ){
                Ok(row) => {
                    extract_user_id_from_row(&row)
                },
                Err(_) => {
                    Err(DatabaseError(QueryFailed(Postgres)))
                }}
        },
    };
    let user_id = match user_id {
        Ok(id) => id,
        Err(e) => {
            return Err(e);
        }
    };
    //TODO check user expired_at
    match postgres.query_one(
        "SELECT is_active FROM user WHERE user_id = $1 AND is_active = true",
        &[&user_id.to_string()]
    ){
        Ok(_)=>{
            return Ok(user_id);
        },
        Err(_)=>{
            return Err(AuthError(AccountSuspended));
        }
    }
}

pub fn generate_response(error:&ErrorKind) -> HttpResponse{
    match error {
       ErrorKind::AuthError(InvalidCredential) => HttpResponse::Unauthorized().body("invalid credential"),
       ErrorKind::AuthError(UserInactive) => HttpResponse::Unauthorized().body("user inactive"),
       ErrorKind::AuthError(AccountSuspended) => HttpResponse::Unauthorized().body("account suspended"),
       ErrorKind::DatabaseError(_)=> HttpResponse::ExpectationFailed().finish(),
    } 
}
