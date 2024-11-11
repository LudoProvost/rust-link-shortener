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

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
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

async fn root_handler() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
            <html lang="en">
            <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Request Code App</title>
        <style>
            body {
                font-family: Arial, sans-serif;
                margin: 20px;
            }
            .container {
                display: flex;
                margin-bottom: 20px;
                align-items: center;
            }
            .input-box {
                margin-right: 10px;
            }
            button {
                margin-left: 5px;
            }
            .response {
                margin-left: 10px;
                width: 200px; /* Adjust width as needed */
            }
        </style>
    </head>
    <body>
        <h1>Request Code App</h1>

        <div class="container">
            <input type="text" id="input1" class="input-box" placeholder="Enter link">
            <button id="button1">Get Code</button>
            <input type="text" id="response1" class="response" placeholder="Response will appear here" readonly>
        </div>

        <div class="container">
            <input type="text" id="input2" class="input-box" placeholder="Enter code">
            <button id="button2">Send Code</button>
            <input type="text" id="response2" class="response" placeholder="Response will appear here" readonly>
        </div>

        <script>
            // Function to handle the first button click
            document.getElementById('button1').addEventListener('click', function () {
                const input1 = document.getElementById('input1').value;
                fetch('http://localhost:8000/link', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({ link: input1 }) // Send input value in the request body
                })
                .then(response => response.json())
                .then(data => {
                    // Assuming the response has a 'code' field
                    document.getElementById('response1').value = data.code || 'No code received';
                })
                .catch(error => {
                    console.error('Error:', error);
                    document.getElementById('response1').value = 'Error fetching code';
                });
            });

            // Function to handle the second button click
            document.getElementById('button2').addEventListener('click', function () {
                const input2 = document.getElementById('input2').value;
                fetch(`http://localhost:8000/${input2}`, {
                    method: 'GET' // Assuming a GET request to fetch the code
                })
                .then(response => response.json())
                .then(data => {
                    // Assuming the response has a 'code' field
                    document.getElementById('response2').value = data.code || 'No code received';
                })
                .catch(error => {
                    console.error('Error:', error);
                    document.getElementById('response2').value = 'Error fetching code';
                });
            });
        </script>
    </body>
    </html>
    "#,
    )
}

// return shortened link
// #[debug_handler]
async fn post_link(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    println!("hi");

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
