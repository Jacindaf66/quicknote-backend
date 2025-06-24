use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use actix_web::HttpMessage;

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub content: String,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct NoteRequest {
    pub title: String,
    pub content: String,
}
