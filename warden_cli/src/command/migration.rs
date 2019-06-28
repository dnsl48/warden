mod build;
mod create;
pub mod list;
mod seal;

use crate::args::Args;
use crate::command;
use crate::MainResult;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(
        name = "create",
        about = "Create a new migration",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    Create { name: String },

    #[structopt(
        name = "build",
        about = "Build migration",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    Build {
        #[structopt(
            long = "force",
            short = "f",
            help = "Rewrite the built migration if exists"
        )]
        force: bool,
        pattern: Option<String>,
    },

    #[structopt(
        name = "list",
        about = "List app migrations",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    List,

    #[structopt(
        name = "seal",
        about = "Seal up a migration",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    Seal {
        #[structopt(
            long = "skip-rebuild",
            short = "s",
            help = "Do not rebuild migration if exists"
        )]
        skip_rebuild: bool,
        pattern: Option<String>,
    }, // #[structopt(name = "")]
}

impl Command {
    pub fn run(&self, args: &Args) -> MainResult {
        match self {
            Command::Create { name } => create::run(args, name),
            Command::Build { force, pattern } => build::run(args, pattern, *force),
            Command::List => list::run(args),
            Command::Seal {
                skip_rebuild,
                pattern,
            } => seal::run(args, pattern, *skip_rebuild),
        }
    }
}

pub fn run(args: Args) -> MainResult {
    match args.command {
        command::Command::Migration { ref command } => command.run(&args),
        _ => unreachable!(),
    }
}
