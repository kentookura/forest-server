use crate::server::*;
use crate::watch::*;
use axum::response::sse::Event;
use miette::Result;
use std::process::exit;
use tokio::sync::broadcast;

pub struct Application {
    port: u16,
    tree_dir: String,
}

impl Application {
    pub fn new(port: u16, dir: String) -> Application {
        Application {
            port,
            tree_dir: dir,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let dir = self.tree_dir.clone();
        let (tx, rx) = broadcast::channel::<Event>(100);

        let backend = async move {
            server(self.port, rx).await;
        };

        let watcher = async {
            let _ = Watcher::run(dir, tx).await;
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
