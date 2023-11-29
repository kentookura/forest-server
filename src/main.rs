use clap::Parser;
use log::info;
mod app;
mod server;
mod watch;

pub use app::*;
pub use server::*;

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
async fn main() -> miette::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    let args = Args::parse();

    (!std::path::Path::new(&args.dir).exists()).then(|| std::process::exit(1));

    pretty_env_logger::init();

    let _app = Application::new(args.port, args.dir).run().await;

    info!("bye");
    Ok(())
}
