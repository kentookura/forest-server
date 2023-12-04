use crate::server::*;
use crate::watch::*;
use axum::response::sse::Event;
use miette::Result;
use std::process::exit;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct Application {
    pub port: u16,
}

impl Application {
    pub async fn run(port: u16, forester_args: &str) -> Result<()> {
        let (tx, rx) = broadcast::channel::<Event>(100);
        let watcher = async {
            let _ = Watcher::run(tx, forester_args).await;
        };

        let backend = async move {
            server(port, Some(rx)).await;
        };

        tokio::select! {
            _ = backend => {}
            _ = watcher => {}
        }

        if tokio::signal::ctrl_c().await.is_ok() {
            exit(0)
        }

        Ok(())
    }
}
