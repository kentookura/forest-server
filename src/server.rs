use axum::{
    extract::State,
    http::header,
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
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
pub struct ServerState {
    rx: Arc<Mutex<Receiver<Event>>>,
}

pub async fn server(port: u16, rx: Option<Receiver<Event>>) {
    info!("Server started, listening on port {}", port);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0:{}", port))
        .await
        .unwrap();

    if let Some(rx) = rx {
        let rx = Arc::new(Mutex::new(rx));
        let state = ServerState { rx };
        let router = Router::new()
            .route("/reload", get(sse_handler))
            .route("/reload.js", get(reloader))
            .nest_service("/", get_service(ServeDir::new("output")))
            .layer(TraceLayer::new_for_http())
            .with_state(state);

        axum::serve(listener, router.into_make_service())
            .await
            .expect("failed to start server")
    } else {
        let router = Router::new()
            .nest_service("/", get_service(ServeDir::new("output")))
            .route("/reload.js", get(notify_no_sse_support));
        axum::serve(listener, router.into_make_service())
            .await
            .expect("failed to start server")
    }
}

#[axum::debug_handler]
async fn sse_handler(State(state): State<ServerState>) -> Sse<BroadcastStream<Event>> {
    let rx: Receiver<Event> = state.rx.lock().unwrap().resubscribe();
    let stream = BroadcastStream::new(rx);
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

async fn reloader() -> impl IntoResponse {
    let content = "
const evtSource = new EventSource(\"reload\");
evtSource.onmessage = (event) => {
    if (event.data == \"reload\") {
      location.reload();
    } else {
      console.log(event.data);
    }
};";

    ([(header::CONTENT_TYPE, "text/javascript")], content)
}

async fn notify_no_sse_support() -> impl IntoResponse {
    let message = "Hot-Reload feature not enabled";
    let content = format!("console.log(\"{}\")", message);
    ([(header::CONTENT_TYPE, "text/javascript")], content)
}
