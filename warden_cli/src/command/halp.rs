use crate::command;
use crate::args::Args;
use crate::MainResult;

use warden_core::dbms;

use structopt::StructOpt;


#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "drivers", about = "Show the list of registered DBMS drivers")]
    Drivers
}


impl Command {
    pub fn run(&self, _args: &Args) -> MainResult {
        match self {
            Command::Drivers => self.drivers()
        }
    }

    fn drivers(&self) -> MainResult {
        println!("Supported drivers:");

        dbms::driver::drivers::<(), _>(|df| {
            println!(" - {}", df.name());
            None
        }).ok();

        Ok(())
    }
}


pub fn run(args: Args) -> MainResult {
    match args.command {
        command::Command::Halp { ref command } => command.run(&args),
        _ => unreachable!()
    }
}
