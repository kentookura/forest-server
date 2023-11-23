use clap::Parser;
use log::info;

mod app;
mod forest;
mod help_line;
mod server;
mod watch;

pub use app::*;
pub use server::*;

use crate::sse::Broadcaster;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The port on which to host the generated files
    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,

    /// Directory containig trees
    #[arg(short, long, default_value = "./trees", value_name = "DIR")]
    pub dir: String,

    /// Directory containig trees
    #[arg(short, long, default_value = "index", value_name = "ROOT-TREE")]
    pub root: String,

    #[arg(short, long, default_value = "false")]
    pub verbose: bool,
}

#[actix_web::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    let args = Args::parse();
    let test = format!("{}/{}", args.dir, args.root);
    log::info!("{:?}", test);
    (!std::path::Path::new(&test).exists()).then(|| {
        log::info!("{:?}", test);
        log::info!("Should I create these directories here?");
    });
    let root = "index"; //TODO: verify paths, ask if we should create them
    let dir = "trees".to_string(); //TODO: verify paths: c
    pretty_env_logger::init();

    let mut rt = tokio::runtime::Runtime::new().unwrap();

    Application::new(args.port, format!("{}/{}.tree", dir, root), dir)
        .run()
        .await;

    info!("bye");
    Ok(())
}
