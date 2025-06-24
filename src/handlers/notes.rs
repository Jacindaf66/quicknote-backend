use actix_web::{
    get, post, put, delete,
    web, HttpRequest, HttpResponse, Responder, Error,
};
use actix_web::error::ErrorInternalServerError;
use actix_web::HttpMessage;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::types::time::OffsetDateTime;

#[derive(Serialize, Deserialize, Debug)]
pub struct Note {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub content: String,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

#[derive(Deserialize)]
pub struct NewNote {
    pub title: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct UpdateNote {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[post("/notes")]
pub async fn create_note(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    new_note: web::Json<NewNote>,
) -> Result<impl Responder, Error> {
    let user_id = req.extensions().get::<i32>().copied().unwrap_or(1);

    let rec = sqlx::query_as!(
        Note,
        r#"
        INSERT INTO notes (user_id, title, content)
        VALUES ($1, $2, $3)
        RETURNING id, user_id, title, content, created_at, updated_at
        "#,
        user_id,
        new_note.title,
        new_note.content
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(ErrorInternalServerError)?;

    Ok(web::Json(rec))
}

#[get("/notes")]
pub async fn list_notes(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> Result<impl Responder, Error> {
    let user_id = req.extensions().get::<i32>().copied().unwrap_or(1);

    let notes = sqlx::query_as!(
        Note,
        r#"
        SELECT id, user_id, title, content, created_at, updated_at
        FROM notes
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(ErrorInternalServerError)?;

    Ok(web::Json(notes))
}

#[get("/notes/{id}")]
pub async fn get_note(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let user_id = req.extensions().get::<i32>().copied().unwrap_or(1);
    let note_id = path.into_inner();

    let note = sqlx::query_as!(
        Note,
        r#"
        SELECT id, user_id, title, content, created_at, updated_at
        FROM notes
        WHERE id = $1 AND user_id = $2
        "#,
        note_id,
        user_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(ErrorInternalServerError)?;

     match note {
        Some(n) => Ok(HttpResponse::Ok().json(n)),  // 统一用 HttpResponse
        None => Ok(HttpResponse::NotFound().json("Note not found")),  // 返回 JSON 格式错误
     }
}

#[put("/notes/{id}")]
pub async fn update_note(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    update: web::Json<UpdateNote>,
) -> Result<impl Responder, Error> {
    let user_id = req.extensions().get::<i32>().copied().unwrap_or(1);
    let note_id = path.into_inner();

    // 检查是否存在
    let existing = sqlx::query!(
        "SELECT id FROM notes WHERE id = $1 AND user_id = $2",
        note_id,
        user_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(ErrorInternalServerError)?;

    if existing.is_none() {
        return Ok(HttpResponse::NotFound().body("Note not found"));
    }

    sqlx::query!(
        r#"
        UPDATE notes SET
            title = COALESCE($1, title),
            content = COALESCE($2, content),
            updated_at = NOW()
        WHERE id = $3 AND user_id = $4
        "#,
        update.title,
        update.content,
        note_id,
        user_id
    )
    .execute(pool.get_ref())
    .await
    .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body("Note updated"))
}

#[delete("/notes/{id}")]
pub async fn delete_note(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let user_id = req.extensions().get::<i32>().copied().unwrap_or(1);
    let note_id = path.into_inner();

    let result = sqlx::query!(
        "DELETE FROM notes WHERE id = $1 AND user_id = $2",
        note_id,
        user_id
    )
    .execute(pool.get_ref())
    .await
    .map_err(ErrorInternalServerError)?;

    if result.rows_affected() == 0 {
        Ok(HttpResponse::NotFound().body("Note not found"))
    } else {
        Ok(HttpResponse::Ok().body("Note deleted"))
    }
}
