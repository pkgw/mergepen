//! Basic Rust-powerd websocket sync server, from the Samod
//! JS compatibility test files.

use futures::lock::Mutex;
use samod::Samod;
use std::{io::Result, sync::Arc};
use tokio::{net::TcpListener, task::JoinHandle};

pub async fn start_server(handle: Samod) -> JoinHandle<Result<()>> {
    let running_connections = Arc::new(Mutex::new(Vec::new()));

    let app = axum::Router::new()
        .route("/", axum::routing::get(websocket_handler))
        .with_state((handle.clone(), running_connections.clone()));

    // NB hardcoded testing port
    let listener = TcpListener::bind("0.0.0.0:20800")
        .await
        .expect("unable to bind socket");

    let server = axum::serve(listener, app).into_future();
    tokio::spawn(server)
}

#[allow(clippy::type_complexity)]
async fn websocket_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    axum::extract::State((handle, running_connections)): axum::extract::State<(
        Samod,
        Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
    )>,
) -> axum::response::Response {
    ws.on_upgrade(|socket| handle_socket(socket, handle, running_connections))
}

async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    repo: Samod,
    running_connections: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
) {
    eprintln!("Accepting websocket connection");
    let driver = repo.accept_axum(socket);
    let handle = tokio::spawn(async {
        let finished = driver.await;
        eprintln!("websocket sync server connection finished: {finished:?}");
    });
    running_connections.lock().await.push(handle);
}
