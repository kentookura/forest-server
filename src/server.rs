use axum::response::Response;
use axum::{
    extract::State,
    http::header,
    http::{
        header::{ACCEPT, CONTENT_TYPE},
        HeaderName, Method, StatusCode,
    },
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    routing::{get, get_service},
    Router,
};
use miette::Result;
use minijinja::Environment;
use std::error;
use std::ffi::OsStr;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::broadcast::Receiver;
use tokio_stream::wrappers::BroadcastStream;
use tower_http::cors::CorsLayer;
use tower_http::{services::ServeDir, trace::TraceLayer};

use tracing::info;
pub fn leak_alloc<T>(value: T) -> &'static T {
    Box::leak(Box::new(value))
}

#[derive(Clone)]
pub struct AppState {
    port: u16,
    rx: Arc<Mutex<Receiver<Event>>>,
}

impl AppState {
    pub fn new(port: u16, rx: Receiver<Event>) -> Self {
        let rx = Arc::new(Mutex::new(rx));
        Self { port, rx }
        //server(port, rx).await;
    }
}

pub struct Server {
    state: AppState,
}

impl Server {
    pub fn new(port: u16, rx: Receiver<Event>) -> Self {
        Self {
            state: AppState {
                port,
                rx: Arc::new(Mutex::new(rx)),
            },
        }
    }
    pub async fn run(&self) -> Result<(), Box<dyn error::Error>> {
        info!("Server started, listening on port {}", self.state.port);

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.state.port))
            .await
            .unwrap();

        let rx = self.state.rx.clone();
        let state = self.state.clone();

        let router = Router::new()
            .route("/reload", get(sse_handler))
            .with_state(state)
            //.merge(route_handler(state))
            //.layer(CacheControlLayer::new())
            .nest_service("/", get_service(ServeDir::new("output")))
            .route("/reload.js", get(reloader));

        axum::serve(
            listener,
            router
                .layer(TraceLayer::new_for_http())
                .layer(
                    CorsLayer::new()
                        .allow_credentials(true)
                        .allow_headers([
                            ACCEPT,
                            CONTENT_TYPE,
                            HeaderName::from_static("csrf-token"),
                        ])
                        .max_age(Duration::from_secs(86400))
                        //.allow_origin(config.cors_origin.parse::<HeaderValue>()?)
                        .allow_methods([
                            Method::GET,
                            Method::POST,
                            Method::PUT,
                            Method::DELETE,
                            Method::OPTIONS,
                            Method::HEAD,
                            Method::PATCH,
                        ]),
                )
                .into_make_service(),
        )
        .await?;
        Ok(())
    }
}

fn import_templates() -> Result<Environment<'static>, Box<dyn error::Error>> {
    let mut env = Environment::new();

    for entry in std::fs::read_dir("templates")?.filter_map(Result::ok) {
        let path = entry.path();

        if path.is_file() && path.extension() == Some(OsStr::new("html")) {
            let name = path
                .file_name()
                .and_then(OsStr::to_str)
                .ok_or("failed to convert path to string")?
                .to_owned();

            let data = std::fs::read_to_string(&path)?;

            env.add_template_owned(name, data)?;
        }
    }

    Ok(env)
}

#[derive(Debug)]
pub enum ApiError {
    TemplateNotFound(String),
    TemplateRender(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status_code, message) = match self {
            Self::TemplateNotFound(template_name) => (
                StatusCode::NOT_FOUND,
                format!("template \"{template_name}\" does not exist"),
            ),
            Self::TemplateRender(template_name) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to render template \"{template_name}\""),
            ),
        };

        (status_code, message).into_response()
    }
}

#[axum::debug_handler]
async fn sse_handler(State(state): State<AppState>) -> Sse<BroadcastStream<Event>> {
    //async fn sse_handler(state: AppState) -> Sse<BroadcastStream<Event>> {
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
