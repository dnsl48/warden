mod app;
// mod halp;
mod migration;

use crate::args::Args;
use crate::MainResult;
use log;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(
        name = "app",
        about = "Control your application",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    App {
        #[structopt(subcommand)]
        command: app::Command,
    },

    #[structopt(
        name = "migration",
        about = "Control your migrations",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    Migration {
        #[structopt(subcommand)]
        command: migration::Command,
    },
    // #[structopt(name = "halp", about = "Get information about warden")]
    // Halp {
    //     #[structopt(subcommand)]
    //     command: halp::Command,
    // },
}

impl Command {
    pub fn run(args: Args) -> MainResult {
        log::debug!(r#"Launching the command "{:?}""#, args.command);
        match args.command {
            Command::App { .. } => self::app::run(args),
            Command::Migration { .. } => self::migration::run(args),
            // Command::Halp { .. } => self::halp::run(args),
        }
    }
}
