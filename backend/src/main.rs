mod auth;
mod db;
mod upload;
mod webrtc;

use axum::{
    extract::{Query, State},
    http::{StatusCode, HeaderMap},
    response::{Html, IntoResponse},
    routing::{get, get_service, patch, post},
    Json, Router,
    middleware::{self, Next},
};
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use futures_util::{stream::StreamExt, sink::SinkExt};
use axum::extract::ws::{WebSocket, WebSocketUpgrade, Message};
use serde::{Deserialize, Serialize};
use headers::{Cookie, HeaderMapExt};

// Structure pour un message de chat
#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChatMessage {
    from: String,
    from_name: String,
    content: String,
    timestamp: i64,
}

// Structure pour suivre une connexion active
struct Connection {
    member_id: String,
    member_name: String,
    sender: tokio::sync::mpsc::UnboundedSender<Message>,
}

type ActiveConnections = Arc<RwLock<HashMap<String, Connection>>>;

#[derive(Clone)]
pub struct SharedState {
    pub db: sqlx::SqlitePool,
    pub webrtc_sessions: Arc<RwLock<HashMap<String, String>>>,
    pub chat_connections: ActiveConnections,
}

// Middleware pour vérifier l'authentification admin
async fn admin_middleware(
    headers: HeaderMap,
    State(state): State<SharedState>,
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Result<axum::response::Response, StatusCode> {
    // Extraire le cookie depuis les headers
    let cookie_header = headers
        .typed_get::<Cookie>()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let admin_token = cookie_header
        .get("nook_admin")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Vérifier si le token admin est valide
    let exists: Option<(i64,)> = sqlx::query_as(
        "SELECT 1 FROM admin_sessions WHERE token = ?"
    )
    .bind(admin_token)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if exists.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}

// Fallback SPA
async fn spa_fallback() -> impl IntoResponse {
    match tokio::fs::read_to_string("/app/static/index.html").await {
        Ok(html) => Html(html),
        Err(_) => Html("<h1>Erreur : index.html introuvable</h1>".to_string()),
    }
}

#[tokio::main]
async fn main() {
    // Création dossiers
    tokio::fs::create_dir_all("/app/data").await.ok();
    tokio::fs::create_dir_all("/app/data/uploads").await.ok();

    println!("Démarrage de Nook v2.0");

    // Token admin (maintenant stocké en base de données)
    println!("Vérification de l'admin...");

    // Init DB
    let app_state = db::init_db().await;
    let shared_state = SharedState {
        db: app_state.db.clone(),
        webrtc_sessions: Arc::new(RwLock::new(HashMap::new())),
        chat_connections: Arc::new(RwLock::new(HashMap::new())),
    };

    // Routes publiques
    let public_routes = Router::new()
        .route("/api/join", post(auth::join_handler))
        .route("/api/login", post(auth::login_handler))
        .route("/api/admin/login", post(auth::admin_login_handler))
        .route("/api/gifs", get(gif_proxy));

    // Routes protégées (utilisateurs normaux)
    let user_routes = Router::new()
        .route("/ws", get(ws_handler))
        .route("/api/upload", post(upload::handle_upload))
        .route("/api/webrtc/offer", post(webrtc::handle_offer))
        .route("/api/webrtc/answer", get(webrtc::handle_answer));

    // Routes admin protégées
    let admin_routes = Router::new()
        .route("/api/admin/invite", post(auth::invite_handler))
        .route("/api/admin/members", get(auth::members_handler))
        .route("/api/admin/members/:id/approve", patch(auth::approve_handler))
        .route("/api/admin/logout", post(auth::admin_logout_handler))
        .route_layer(middleware::from_fn_with_state(
            shared_state.clone(),
            admin_middleware,
        ));

    let app = Router::new()
        .merge(public_routes)
        .merge(user_routes)
        .merge(admin_routes)
        // Assets statiques
        .nest_service("/_app", get_service(ServeDir::new("/app/static/_app")))
        .nest_service("/static", get_service(ServeDir::new("/app/static")))
        .nest_service("/uploads", get_service(ServeDir::new("/app/data/uploads")))
        // Fallback SPA
        .fallback(get(spa_fallback))
        .with_state(shared_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Nook prêt sur http://{}", addr);
    println!("Static files : /app/static");
    println!("Uploads : /app/data/uploads");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

// Handler WebSocket avec cookie de session
async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    // Extraire le cookie de session depuis les headers
    let cookie_header = match headers.typed_get::<Cookie>() {
        Some(cookie) => cookie,
        None => return (StatusCode::UNAUTHORIZED, "Session requise").into_response(),
    };

    let session_token = match cookie_header.get("nook_session") {
        Some(token) => token,
        None => return (StatusCode::UNAUTHORIZED, "Session invalide").into_response(),
    };

    match auth::validate_session_and_get_user(&state.db, session_token).await {
        Ok((member_id, member_name)) => {
            // Accepte la mise à niveau WebSocket
            ws.on_upgrade(move |socket| handle_authenticated_connection(socket, state, member_id, member_name))
        }
        Err(_) => (StatusCode::UNAUTHORIZED, "Session expirée ou invalide").into_response(),
    }
}

// Gère une connexion WebSocket authentifiée
async fn handle_authenticated_connection(
    socket: WebSocket,
    state: SharedState,
    member_id: String,
    member_name: String,
) {
    let (mut sender, mut receiver) = socket.split();

    // Création d'un canal pour envoyer des messages à ce client
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    // Stockage de la connexion
    let conn = Connection {
        member_id: member_id.clone(),
        member_name: member_name.clone(),
        sender: tx,
    };
    state
        .chat_connections
        .write()
        .await
        .insert(member_id.clone(), conn);

    // Tâche pour recevoir les messages du client et les diffuser
    let state_for_receive = state.clone();
    let member_id_for_receive = member_id.clone();
    let member_name_for_receive = member_name.clone();
    tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                // Construire un message structuré
                let chat_msg = ChatMessage {
                    from: member_id_for_receive.clone(),
                    from_name: member_name_for_receive.clone(),
                    content: text,
                    timestamp: chrono::Utc::now().timestamp(),
                };
                // Diffusion à tous les autres utilisateurs
                broadcast_message(&state_for_receive, &chat_msg).await;
            }
        }
        // Déconnexion : retirer de la liste des connexions actives
        state_for_receive
            .chat_connections
            .write()
            .await
            .remove(&member_id_for_receive);
    });

    // Tâche pour envoyer les messages du canal au client WebSocket
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break; // La connexion est fermée
            }
        }
    });
}

// Fonction pour diffuser un message à toutes les connexions actives (sauf l'expéditeur)
async fn broadcast_message(state: &SharedState, message: &ChatMessage) {
    let connections = state.chat_connections.read().await;
    let json_msg = serde_json::to_string(message).unwrap();

    for (member_id, connection) in connections.iter() {
        // Ne pas renvoyer le message à l'expéditeur
        if *member_id != message.from {
            let _ = connection.sender.send(Message::Text(json_msg.clone()));
        }
    }
}

// GIF proxy
use urlencoding::encode;

async fn gif_proxy(Query(params): Query<HashMap<String, String>>) -> Result<Json<Value>, StatusCode> {
    if let Some(q) = params.get("q") {
        let url = format!(
            "https://g.tenor.com/v1/search?q={}&key=LIVDSRZULELA&limit=8",
            encode(q)
        );
        let resp = reqwest::get(&url)
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)?;
        let json: Value = resp.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
        Ok(Json(json))
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}
