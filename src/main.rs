pub use app::*;
use clap::{arg, value_parser, Command};
pub use server::*;
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
            Command::new("serve").arg(
                arg!(port: [PORT])
                    .value_parser(value_parser!(u16))
                    .default_value("8080"),
            ),
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
                .unwrap();

            let Some(port) = sub_matches.get_one::<u16>("port") else {
                todo!()
            };

            let _app = Application::run(*port, opts).await;
        }
        Some(("serve", sub_matches)) => {
            let Some(port) = sub_matches.get_one::<u16>("port") else {
                todo!()
            };

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
