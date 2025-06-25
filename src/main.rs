// mod handlers;
// mod middleware;

// use actix_web::{App, HttpServer, web};
// use actix_cors::Cors;
// use sqlx::PgPool;
// use handlers::{register, login};
// use dotenv::dotenv;
// use crate::handlers::notes;
// use middleware::auth::AuthMiddleware;

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     dotenv().ok();  // 加载 .env

//     let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     let pool = PgPool::connect(&database_url).await.expect("无法连接数据库");
//     let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "my_secret".to_string());


//     println!("Starting server at http://localhost:8080");

//     HttpServer::new(move || {
//         App::new()
//             .wrap(
//                 Cors::default()
//                     .allow_any_origin()
//                     .allow_any_method()
//                     .allow_any_header()
//             )
//             .wrap(AuthMiddleware::new(jwt_secret.clone()))
//             .app_data(web::Data::new(pool.clone()))
//             .service(register)
//             .service(login)
//             .service(notes::create_note)
//             .service(notes::list_notes)
//             .service(notes::get_note)
//             .service(notes::update_note)
//             .service(notes::delete_note)
//     })
//     .bind(("127.0.0.1", 8080))?
//     .run()
//     .await
// }

mod handlers;
mod middleware;
mod models;

use actix_web::{App, HttpServer, web};
use actix_cors::Cors;
use sqlx::PgPool;
use handlers::{register, login};
use dotenv::dotenv;
use crate::handlers::notes;
use middleware::auth::AuthMiddleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); // 加载 .env 文件

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("无法连接数据库");

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "my_secret".to_string());

    println!("🚀 Server running at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default() // ✅ 全局启用 CORS
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
            )
            .app_data(web::Data::new(pool.clone()))
            // ✅ 不需要认证的接口直接注册
            .service(register)
            .service(login)
            // ✅ 把需要认证的接口统一放到 /api 下，并加上 AuthMiddleware
            .service(
                web::scope("/api")
                    .wrap(AuthMiddleware::new(jwt_secret.clone()))
                    .service(notes::create_note)
                    .service(notes::list_notes)
                    .service(notes::get_note)
                    .service(notes::update_note)
                    .service(notes::delete_note)
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
