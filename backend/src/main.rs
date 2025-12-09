use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    std::fs::create_dir_all("data").ok();
    let token_path = "data/admin.token";
    if !std::path::Path::new(token_path).exists() {
        let token = uuid::Uuid::new_v4().to_string();
        std::fs::write(token_path, token).expect("Failed to create admin.token");
        println!("✅ Admin token created: data/admin.token");
    }

    // Serve frontend files from /static at runtime
    let app = Router::new()
        .route("/", get(|| async { "" })) // fallback to index.html
        .route("/admin", get(|| async { "" })) // fallback to admin.html
        .nest_service("/static", ServeDir::new("static"))
        .fallback_service(ServeFile::new("static/index.html"));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Nook running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}