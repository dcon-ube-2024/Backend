use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool, Sqlite};
use std::env;
use dotenvy::dotenv;
use tokio::sync::OnceCell;

static POOL: OnceCell<SqlitePool> = OnceCell::const_new();

pub async fn init() {
    dotenv().expect(".env file not found");
    let database_url = env::vars()
        .find(|(key, _)| key == "DATABASE_URL")
        .expect("DATABASE_URL not found")
        .1;

    if !Sqlite::database_exists(&database_url)
        .await
        .expect("Failed to check database exists.")
    {
        Sqlite::create_database(&database_url)
            .await
            .expect("Falied to create database.");
    }

    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed connect to database.");

    sqlx::migrate!("db/migrations")
        .run(&pool)
        .await
        .inspect_err(|e| eprintln!("{e}"))
        .expect("Failed to migrate database.");

    POOL.set(pool).expect("Failed to set connection pool");
}


pub async fn get_pool() -> Result<SqlitePool, &'static str> {
    POOL.get().cloned().ok_or("Failed to get connection pool.")
}