mod create;
mod deploy;
mod status;

use crate::args::Args;
use crate::command;
use crate::MainResult;
use warden_core::path::relpath;

use log;
use path_abs::{PathArc, PathDir};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(
        name = "create",
        about = "Create a new application",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    Create {
        driver: String,

        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },

    #[structopt(
        name = "deploy",
        about = "Deploy the application",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    Deploy,

    #[structopt(
        name = "status",
        about = "Prints warden status",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    Status,
}

impl Command {
    pub fn run(&self, args: &Args) -> MainResult {
        match self {
            Command::Create { driver, path } => self.create(driver, path.as_path()),
            Command::Deploy => deploy::run(args),
            Command::Status => status::run(args),
        }
    }

    fn create(&self, driver: &str, path: &Path) -> MainResult {
        let path = PathDir::create(PathArc::new(path).absolute()?)?;
        log::debug!("Creating app at: {:?}", relpath(&path)?);
        create::init(driver, path)?;

        Ok(())
    }
}

pub fn run(args: Args) -> MainResult {
    match args.command {
        command::Command::App { ref command } => command.run(&args),
        _ => unreachable!(),
    }
}
