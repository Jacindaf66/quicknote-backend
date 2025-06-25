use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use argon2::{Argon2, PasswordHasher, PasswordHash, PasswordVerifier};
use argon2::password_hash::SaltString;
use rand_core::OsRng;
use jsonwebtoken::{encode, EncodingKey, Header};
use chrono::{Utc, Duration};
use std::env;
use crate::middleware::claims::Claims;



#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[post("/register")]
pub async fn register(
    pool: web::Data<PgPool>,
    form: web::Json<RegisterRequest>,
) -> impl Responder {
    if form.username.is_empty() || form.password.is_empty() {
        return HttpResponse::BadRequest().body("用户名和密码不能为空");
    }

    // 生成随机盐
    let salt = SaltString::generate(&mut OsRng);
    // Argon2 默认配置
    let argon2 = Argon2::default();

    // 计算密码哈希
    let password_hash = match argon2.hash_password(form.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(_) => return HttpResponse::InternalServerError().body("密码加密失败"),
    };

    let result = sqlx::query!(
        "INSERT INTO users (username, password_hash) VALUES ($1, $2)",
        form.username,
        password_hash
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("注册成功"),
        Err(e) => {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.constraint() == Some("users_username_key") {
                    return HttpResponse::BadRequest().body("用户名已存在");
                }
            }
            HttpResponse::InternalServerError().body("数据库错误")
        }
    }
}


#[post("/login")]
pub async fn login(
    pool: web::Data<PgPool>,
    form: web::Json<LoginRequest>,
) -> impl Responder {
    let row = sqlx::query!(
      "SELECT id, password_hash FROM users WHERE username = $1",
       form.username
    )

    .fetch_optional(pool.get_ref())
    .await;

    let row = match row {
        Ok(Some(record)) => record,
        Ok(None) => return HttpResponse::Unauthorized().body("用户名或密码错误"),
        Err(_) => return HttpResponse::InternalServerError().body("数据库错误"),
    };

    let parsed_hash = PasswordHash::new(&row.password_hash);
    let verify_result = parsed_hash
        .ok()
        .and_then(|ph| {
            Argon2::default()
                .verify_password(form.password.as_bytes(), &ph)
                .ok()
        });

    if verify_result.is_none() {
        return HttpResponse::Unauthorized().body("用户名或密码错误");
    }

    // 生成 JWT
    // let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "my_secret".into());
    // let claims = Claims {
    //     sub: form.username.clone(),
    //     exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
    // };
     // ✅ 生成 JWT，使用 user_id 而非用户名
     let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "my_secret".into());
    // let claims = Claims {
    //     user_id: row.id,  // <<-- 使用数据库返回的 id
    //     exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
    // };
    let claims = Claims {
    user_id: row.id,             // 使用查询结果中的 id
    sub: form.username.clone(),   // 这里一般放用户名或者用户id的字符串
    exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
    
};


    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .unwrap_or_else(|_| "token_error".into());

    HttpResponse::Ok().body(token)
}
