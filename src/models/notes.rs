use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use actix_web::HttpMessage;
use sqlx::types::time::OffsetDateTime;

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

#[derive(Debug, Deserialize)]
pub struct ListNotesQuery {
    pub user_id: Option<i32>,  // 按用户ID筛选（可选）
    pub keyword: Option<String>,  // 按关键词搜索标题/内容（可选）
    pub limit: Option<i64>,     // 分页限制
    pub offset: Option<i64>,   // 分页偏移
}