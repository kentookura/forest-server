pub use app::*;
use axum::response::sse::Event as SseEvent;
use clap::{arg, value_parser, Command};
use log::{debug, error, info};
pub use logging::*;
pub use logging::{LogCollector, LogMessage};
use miette::Result;
use parking_lot::Mutex;
pub use server::*;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_log::LogTracer;
use tracing_subscriber::{prelude::*, EnvFilter, Registry, filter::LevelFilter };
extern crate watchexec as wx;

mod app;
mod events;
mod home;
mod logging;
mod root;
mod server;
mod ui;
mod watch;
mod widgets;

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
    let (logs, _guard) = setup_logging()?;

    if which::which("forester").is_err() {
        error!("Forester is not installed");
        info!("Please install it with `opam install forester` or `nix shell sourcehut:~jonsterling/ocaml-forester`");
        exit(1);
    }

    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("watch", sub_matches)) => {
            if sub_matches
                .get_one::<String>("opts")
                .map(|s| s.as_str())
                .is_none()
            {
                error!("Make sure to pass arguments to forester like so: forest watch -- \"build --dev --index root trees\"");
                exit(1)
            };

            let opts = sub_matches
                .get_one::<String>("opts")
                .map(|s| s.as_str())
                .unwrap();

            let Some(port) = sub_matches.get_one::<u16>("port") else {
                todo!()
            };

            //let _app = app::run(*port, logs, opts).await;
            app::run(*port, logs, opts).await;
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

            let (tx, rx) = broadcast::channel::<SseEvent>(100);
            let server = Server::new(*port, rx);
            server.run().await;
            //let _server = server::run(*port, None).await;
        }
        Some(("publish", _)) => {
            println!("Not implemented yet.");
            println!("Coming soon to a forest near you!")
        }
        _ => unreachable!(),
    }
    Ok(())
}

/// Sets up logging to a file and a collector for the logs that can be used to
/// display them in the UI.
fn setup_logging() -> Result<(Arc<Mutex<Vec<LogMessage>>>, WorkerGuard)> {
    // handle logs from the log crate by forwarding them to tracing
    LogTracer::init();

    // handle logs from the tracing crate
    let log_folder = xdg::BaseDirectories::with_prefix("forest-server")
        .expect("failed to get XDG base directories")
        .get_state_home();
    let file_appender = tracing_appender::rolling::hourly(log_folder, "forest-server.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::default()
        .add_directive("hyper=info".parse().unwrap())
        .add_directive("debug".parse().unwrap());

    // collect logs in a collector that can be used to display them in the UI
    let log_collector = LogCollector::default();
    let logs = log_collector.logs();

    let subscriber = Registry::default()
        .with(log_collector.with_filter(LevelFilter::INFO));

    
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    Ok((logs, guard))
}
