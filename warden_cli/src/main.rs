mod args;
mod command;
pub mod grid;
mod migration;

use dotenv::dotenv;
use env_logger;
use exitfailure::ExitFailure;
use structopt::StructOpt;

use args::Args;
use command::Command;

use log;

pub type MainResult = Result<(), ExitFailure>;

fn main() -> MainResult {
    #[cfg(feature = "with-postgres")]
    {
        warden_postgres::driver::register_driver();
    }

    let args = Args::from_args();

    env_logger::builder()
        .default_format_timestamp(false)
        .filter_level(args.get_log_level_filter())
        .init();

    log::set_max_level(args.get_log_level_filter());

    log::trace!("Verbosity: {}", args.verbosity);
    log::trace!("Log level is: {:?}", args.get_log_level_filter());
    log::trace!("Static log level is: {:?}", log::STATIC_MAX_LEVEL);

    if args.dotenv {
        log::trace!("Loading dotenv");
        let path = dotenv()?;
        log::info!("Dotenv: {:?}", path);
    }

    if let Some(ref path) = args.config {
        log::info!("Config: {:?}", path);
    } else if let Ok(path) = ::std::env::var("WARDEN_CONFIG_FILE") {
        log::info!("Config: {}", path);
    } else {
        log::info!("No config defined");
    }

    Command::run(args)
}
