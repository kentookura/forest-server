pub use app::*;
use clap::Parser;
use log::{error, info};
pub use server::*;
mod app;
mod server;
mod watch;

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
    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();

    let args = Args::parse();

    (!std::path::Path::new(&args.dir).exists()).then(|| {
        error!(
            "{} does not exist. Specify a directory containing your trees with --dir",
            args.dir
        );
        std::process::exit(1)
    });

    let _app = Application::new(args.port, args.dir).run().await;

    info!("bye");
    Ok(())
}
