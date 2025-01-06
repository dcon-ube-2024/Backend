use actix_web::{App, HttpServer};
use actix_cors::Cors;
use env_logger::Env;
use anyhow::Ok;
use dotenvy::dotenv;

mod database;
mod handlers;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    database::init().await;
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    HttpServer::new(|| {
        App::new()
        .wrap(
            Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
        )
        .configure(|cfg| { handlers::configure(cfg); })
    })
    .bind(("0.0.0.0", 3001))?
    .run()
    .await?;
    
    Ok(())
}
