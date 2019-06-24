use failure::Error;
use log::LevelFilter;
use path_abs::PathArc;
use structopt::StructOpt;
use std::path::PathBuf;
use std::ffi::OsStr;

use crate::command::Command;
use warden_core::path;

use warden_core::config::Config;


fn os_path(path: &OsStr) -> PathBuf {
    let path = PathBuf::from(path);
    match path::normalise(&path).map(|p| p.as_path().to_path_buf()) {
        Ok(path) => path,
        Err(_) => path
    }
}


#[derive(Debug, StructOpt)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
pub struct Args {
    #[structopt(long = "dotenv", help = "Find and read a dotenv file")]
    pub dotenv: bool,

    #[structopt(long = "config", help = "Path to the configuration file", parse(from_os_str = "os_path"))]
    pub config: Option<PathBuf>,

    #[structopt(
        long = "verbosity",
        short = "v",
        help = "Verbosity, `-v` warnings, `-vv` info, `-vvv` debug, and `-vvvv` trace",
        parse(from_occurrences)
    )]
    pub verbosity: u8,

    #[structopt(subcommand)]
    pub command: Command
}

impl Args {
    pub fn get_log_level_filter(&self) -> LevelFilter {
        match self.verbosity {
            0 => LevelFilter::Error,
            1 => LevelFilter::Warn,
            2 => LevelFilter::Info,
            3 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    }

    pub fn get_config(&self) -> Result<Config, Error> {
        let cfg_path = self.config.as_ref().map(|path| PathArc::new(path));
        let config = Config::new(&cfg_path)?;

        Ok(config)
    }
}
