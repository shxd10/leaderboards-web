use axum::serve;
use routes::router;

mod routes;
mod response;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:3000";

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("Server running on {addr:?}");

    serve(listener, router()).await.unwrap();
}
