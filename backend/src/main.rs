use axum::{
    routing::{get},
    Router, response::Html,
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    // Créer le dossier data si absent
    std::fs::create_dir_all("data").ok();

    // Vérifier/générer admin.token
    let token_path = "data/admin.token";
    if !std::path::Path::new(token_path).exists() {
        let token = uuid::Uuid::new_v4().to_string();
        std::fs::write(token_path, token).expect("Failed to create admin.token");
        println!("✅ Admin token created: data/admin.token");
        println!("👉 Only someone with access to this file can configure the admin interface.");
    }

    let app = Router::new()
        .route("/", get(|| async { Html(include_str!("../../static/index.html")) }))
        .route("/admin", get(|| async { Html(include_str!("../../static/admin.html")) }))
        .nest_service("/static", ServeDir::new("static"))
        .fallback_service(ServeDir::new("static").append_index_html_on_directories(true));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Nook running on http://{}", addr);

    // Utiliser `axum::serve` + `hyper_util::rt::TokioIo`
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service()
    ).await.unwrap();
}