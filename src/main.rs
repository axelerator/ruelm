use axum::Router;
use std::path::PathBuf;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    let www_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("www");

    let app = Router::new()
        .route_service("/", ServeFile::new(www_dir.join("index.html")))
        .nest_service("/assets", ServeDir::new(www_dir.join("assets")));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
