use failure::Error;
use libflate::gzip::Decoder;
use path_abs::PathDir;
use tar;
use warden_core::dbms::driver::{Driver, DriverFactory};
use warden_core::dbms::Connection;

static NAME: &'static str = "postgresql";

pub struct PostgreSQL;

impl Driver for PostgreSQL {
    fn name(&self) -> &'static str { NAME }

    fn create_initial_migration(&self, folder: &PathDir) -> Result<(), Error> {
        Ok(tar::Archive::new(Decoder::new(crate::zero_migration_tar())?).unpack(folder)?)
    }

    fn open_connection(&self, url: &str) -> Result<Box<Connection>, Error> {
        Ok(Box::new(super::connection::open(url)?))
    }
}


pub struct Factory;

impl DriverFactory for Factory {
    fn name(&self) -> &'static str { NAME }

    fn new(&self) -> Box<Driver> {
        Box::new(PostgreSQL)
    }
}

pub fn register_driver() {
    warden_core::dbms::driver::register_driver(Box::new(Factory));
}
