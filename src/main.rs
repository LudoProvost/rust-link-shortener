use axum::{
    body::Body,
    debug_handler,
    extract::{Json, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use rand::{distributions::Alphanumeric, Rng};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

#[derive(Clone)]
struct AppState {
    arr: Arc<Mutex<Vec<Links>>>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        arr: Arc::new(Mutex::new(Vec::new())),
    };

    let router = Router::new()
        .route("/", get(root_handler))
        .route("/link", post(post_link))
        .route("/:shortened", get(get_short_link))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    let tcp = TcpListener::bind(&addr).await.unwrap();

    axum::serve(tcp, router).await.unwrap();
}

#[derive(Clone)]
struct Links {
    long_link: String,
    short_link: String,
}

fn generate_rand_code(length: u32) -> String {
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| rng.sample(Alphanumeric))
        .map(char::from)
        .collect()
}

async fn root_handler() -> Html<String> {
    println!("root.\n");
    let mut file = match File::open("static/index.html").await {
        Ok(f) => f,
        Err(_) => return Html("Error loading page".to_string()),
    };

    let mut contents = String::new();
    if let Err(_) = file.read_to_string(&mut contents).await {
        return Html("Error reading file".to_string());
    }

    Html(contents)
}

// return shortened link
// #[debug_handler]
async fn post_link(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    println!("shortening link.\n");

    let code = generate_rand_code(4);

    let long_link_received = payload.get("link").and_then(|v| v.as_str()).unwrap_or("");

    // create new Links object with code and long_link + find a way to save it (use with_state())
    let temp = Links {
        long_link: long_link_received.to_string(),
        short_link: code.clone(),
    };

    let mut arr = state.arr.lock().unwrap();
    arr.push(temp);

    (StatusCode::OK, Json(json!({ "code": code }))).into_response()
}

// redirects the user to the link associated with the shortened link
async fn get_short_link(State(state): State<AppState>, Path(shortened): Path<String>) -> Response {
    println!("redirecting.\n");
    let arr_lock = state.arr.lock();

    match arr_lock {
        Ok(arr) => {
            if let Some(link) = arr.iter().find(|l| l.short_link == shortened) {
                return Redirect::permanent(&link.long_link).into_response();
            }
        }
        Err(_) => {
            return (StatusCode::NOT_FOUND, Json(json!({ "code": "" }))).into_response();
        }
    }

    (StatusCode::NOT_FOUND, Json(json!({ "code": "" }))).into_response()
}
