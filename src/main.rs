use std::net::SocketAddr;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};

#[derive(Clone)]
struct AppState {
    counter: i32,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to read .env file");
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user))
        .route("/register", get(register))
        .route("/connect", get(connect))
        .layer(Extension(AppState { counter: 0 }))
        .layer(CookieManagerLayer::new());

    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));

    tracing::debug!("listening on {addr}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn connect(
    ws: WebSocketUpgrade,
    cookies: Cookies,
    Extension(state): Extension<AppState>,
) -> Response {
    println!("Cookies: {:?}", cookies.list());
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "message", rename_all = "lowercase")]
enum MessageIncoming {
    Welcome { hello: String },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "message", rename_all = "lowercase")]
enum MessageOutgoing {
    Thanks { name: String },
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(text) => {
                    if let Ok(message) = serde_json::from_str::<MessageIncoming>(&text) {
                        match message {
                            MessageIncoming::Welcome { hello } => {
                                let answer = MessageOutgoing::Thanks { name: hello };
                                if let Ok(str) = serde_json::to_string(&answer) {
                                    if socket.send(Message::Text(str)).await.is_err() {
                                        // client disconnected
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
                Message::Close(close_frame) => {
                    println!("message close {:?}", close_frame);
                }
                _ => {}
            };
        } else {
            // client disconnected
            return;
        };
    }
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn register(cookies: Cookies) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4();
    let mut cookie = Cookie::new("token", uuid::Uuid::new_v4().to_string());
    cookie.set_http_only(Some(true));
    cookies.add(cookie);

    Json(User {
        id: id.as_u64_pair().0,
        username: "Demo".to_string(),
    })
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
