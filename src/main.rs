pub use app::*;
pub use server::*;

use clap::{arg, value_parser, Command};
use log::{debug, error, info};
use std::path::PathBuf;
use std::sync::Arc;
extern crate watchexec as wx;

mod app;
mod server;
mod watch;

fn cli() -> Command {
    Command::new("forest")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("watch")
                .arg(
                    arg!(port: [PORT])
                        .value_parser(value_parser!(u16))
                        .default_value("8080"),
                )
                .arg(arg!(opts: [OPTS]).last(true)),
        )
        .subcommand(
            Command::new("serve")
                .arg(
                    arg!(port: [PORT])
                        .value_parser(value_parser!(u16))
                        .default_value("8080"),
                )
                .arg(arg!(opts: [OPTS]).last(true)),
        )
        .subcommand(Command::new("publish"))
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();

    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("watch", sub_matches)) => {
            let opts = sub_matches
                .get_one::<String>("opts")
                .map(|s| s.as_str())
                .expect("make sure to pass arguments to forester like so: forest watch -- \"build --dev --index root trees/\"");

            let Some(port) = sub_matches.get_one::<u16>("port") else {
                todo!()
            };

            let _app = Application::run(*port, opts).await;
        }
        Some(("serve", sub_matches)) => {
            let opts = sub_matches.get_one::<String>("opts").map(|s| s.as_str());
            let mut forester_args = "build";
            if let Some(opts) = opts {
                forester_args = opts;
            }

            let Some(port) = sub_matches.get_one::<u16>("port") else {
                todo!()
            };

            let args = shlex::split(forester_args).expect("failed to parse arguments for forester");

            let forester = Arc::new(wx::command::Command {
                program: wx::command::Program::Exec {
                    prog: PathBuf::from("forester"),
                    args,
                },
                options: Default::default(),
            });

            let output = forester.to_spawnable().output().await.map_err(|e| {
                error!("Failed to call forester: {}", e);
                info!("Attempting to serve directory \"output\" anyway.");
            });

            if let Ok(output) = output {
                info!("Build Succeeded");
                let stdout = String::from_utf8(output.stdout).expect("Output not UTF8");
                debug!("{}", stdout);
            }

            let _server = server(*port, None).await;
        }
        Some(("publish", _)) => {
            println!("Not implemented yet.");
            println!("Coming soon to a forest near you!")
        }
        _ => unreachable!(),
    }
    Ok(())
}
