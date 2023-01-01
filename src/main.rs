use std::net::SocketAddr;

use axum::{
    // See https://docs.rs/axum/latest/axum/extract/ws/index.html
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::StatusCode,
    Json,
    response::IntoResponse, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to read .env file");
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user))
        .route("/register", get(register))
        .layer(CookieManagerLayer::new());

    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));

    tracing::debug!("listening on {addr}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn register(cookies: Cookies) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4();
    let mut cookie = Cookie::new("token", uuid::Uuid::new_v4().to_string());
    cookie.set_http_only(Some(true));
    cookies.add(cookie);

    Json(User { id: id.as_u64_pair().0, username: "Demo".to_string() })
}

#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}

async fn create_user(Json(create_user_input): Json<CreateUser>) -> impl IntoResponse {
    let user = User {
        id: 13337,
        username: create_user_input.username,
    };

    (StatusCode::CREATED, Json(user))
}
