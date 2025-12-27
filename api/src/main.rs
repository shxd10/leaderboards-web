use sqlx::SqlitePool;
use std::env;

mod response;
mod routes;
mod models;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenvy::dotenv().ok();

    let addr = env::var("LOCALHOST").expect("LOCALHOST must be set");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = SqlitePool::connect(&database_url).await?;

    let listener = tokio::net::TcpListener::bind(addr.clone()).await?;

    axum::serve(listener, routes::router(pool).into_make_service()).await?;

    println!("Server running on {addr:?}");

    Ok(())
}
