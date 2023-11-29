use axum::{
    extract::State,
    response::sse::{Event, Sse},
    routing::{get, get_service},
    Router,
};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::broadcast::Receiver;
use tokio_stream::wrappers::BroadcastStream;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::info;

#[derive(Clone)]
struct AppState {
    rx: Arc<Mutex<Receiver<Event>>>,
}

pub async fn server(port: u16, rx: Receiver<Event>) {
    let rx = Arc::new(Mutex::new(rx));
    let state = AppState { rx };
    let app = Router::new()
        .route("/reload", get(sse_handler))
        .nest_service("/", get_service(ServeDir::new("output")))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    info!("Server started, listening on port {}", port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0:{}", port))
        .await
        .unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .expect("failed to start server")
}

async fn sse_handler(State(state): State<AppState>) -> Sse<BroadcastStream<Event>> {
    let rx: Receiver<Event> = state.rx.lock().unwrap().resubscribe();
    let stream = BroadcastStream::new(rx);
    info!("sse handler called");

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
