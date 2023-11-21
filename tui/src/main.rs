use anyhow::{bail, Result};
use clap::Parser;
use std::process::exit;

mod app;
mod build;
mod help_line;
mod server;

pub use app::*;
pub use build::*;
pub use server::*;

#[macro_use]
extern crate cli_log;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_cli_log!();
    let args = Args::parse();
    let test = format!("{}/{}", args.dir, args.root);
    (!std::path::Path::new(&test).exists()).then(|| {
        log::info!("{:?}", test);
        log::info!("Should I create these dirs here?");
    });
    let root = "index"; //TODO: verify paths, ask if we should create them
    let dir = "trees".to_string(); //TODO: verify paths: c
    if args.verbose {
        std::env::set_var("RUST_LOG", "debug");
    };
    pretty_env_logger::init();

    App::new(args.port, format!("{}/{}.tree", dir, root), dir)
        .run()
        .await?;

    info!("bye");
    Ok(())
}
