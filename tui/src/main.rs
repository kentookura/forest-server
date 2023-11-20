use anyhow::{bail, Result};
use clap::Parser;
use std::process::exit;

mod app;
mod cli;
mod help_line;
mod server;

pub use app::*;
pub use cli::*;
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
    pub trees: String,

    #[arg(short, long, default_value = "false")]
    pub verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_cli_log!();
    let args = Args::parse();

    //cmd.error(
    //    clap::error::ErrorKind::DisplayHelp,
    //    format!(
    //    "{}\n{}",
    //    msg,
    //    "Specify a path using --trees=<DIR>. You can use patterns such as 'forests/*'"
    //),
    //)
    //.exit();

    match cli::verify() {
        Err(err) => {
            eprintln!("{:?}", err);
            exit(0)
        }
        Ok(args) => {
            if args.verbose {
                std::env::set_var("RUST_LOG", "debug");
            };
            pretty_env_logger::init();
            App::new(args).run().await?;
        }
    }

    info!("bye");
    Ok(())
}
