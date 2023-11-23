use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::{Message, Session};
use futures::stream::{FuturesUnordered, StreamExt};
use log::{debug, info};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Reload {
    inner: Arc<Mutex<ReloadInner>>,
}

struct ReloadInner {
    sessions: Vec<Session>,
}

impl Reload {
    pub fn new() -> Self {
        Reload {
            inner: Arc::new(Mutex::new(ReloadInner {
                sessions: Vec::new(),
            })),
        }
    }

    pub async fn insert(&self, session: Session) {
        self.inner.lock().await.sessions.push(session);
    }

    pub async fn send(&self, msg: String) {
        let mut inner = self.inner.lock().await;
        let mut unordered = FuturesUnordered::new();

        for mut session in inner.sessions.drain(..) {
            let msg = msg.clone();
            unordered.push(async move {
                let res = session.text(msg).await;
                res.map(|_| session).map_err(|_| info!("Dropping session"))
            });
        }

        while let Some(res) = unordered.next().await {
            if let Ok(session) = res {
                inner.sessions.push(session);
            }
        }
    }
}

pub async fn ws(
    req: HttpRequest,
    body: web::Payload,
    reloader: web::Data<Reload>,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, mut session, mut stream) = actix_ws::handle(&req, body)?;

    reloader.insert(session.clone()).await;
    debug!("Inserted session");

    let alive = Arc::new(Mutex::new(Instant::now()));

    let mut session2 = session.clone();
    let alive2 = alive.clone();

    actix_rt::spawn(async move {
        let mut interval = actix_rt::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            if session2.ping(b"").await.is_err() {
                break;
            }

            if Instant::now().duration_since(*alive2.lock().await) > Duration::from_secs(10) {
                let _ = session2.close(None).await;
                break;
            }
        }
    });

    actix_rt::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }
                Message::Text(s) => {
                    info!("Relaying text, {}", s);
                    let s: &str = s.as_ref();
                    reloader.send(s.into()).await;
                }
                Message::Close(reason) => {
                    let _ = session.close(reason).await;
                    info!("Got close, bailing");
                    return;
                }
                Message::Continuation(_) => {
                    let _ = session.close(None).await;
                    info!("Got continuation, bailing");
                    return;
                }
                Message::Pong(_) => {
                    *alive.lock().await = Instant::now();
                }
                _ => (),
            };
        }
        let _ = session.close(None).await;
    });
    info!("Spawned");

    Ok(response)
}
