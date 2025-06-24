mod handlers;

use actix_web::{App, HttpServer, web};
use sqlx::PgPool;
use handlers::{register, login};
use dotenv::dotenv;
use crate::handlers::notes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();  // 加载 .env

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await.expect("无法连接数据库");

    println!("Starting server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(register)
            .service(login)
            .service(notes::create_note)
            .service(notes::list_notes)
            .service(notes::get_note)
            .service(notes::update_note)
            .service(notes::delete_note)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
