use actix_multipart::form::{MultipartForm, json::Json, tempfile::TempFile};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Post {
    pub post_id: Uuid,
    pub title: String,
    pub creator: String,
    pub source: String,
    pub description: String,
    pub uploader: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseFile{
    pub file: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadJson {
    pub title: String,
    pub creator: String,
    pub source: String,
    pub description: String,
}

#[derive(Debug, MultipartForm)]
pub struct UploadFrom {
    #[multipart(limit = "10MB")]
    pub file: Vec<TempFile>,
    pub metadata: Json<Vec<UploadJson>>,
}
