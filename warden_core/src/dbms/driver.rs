use path_abs::PathDir;
use failure::Error;
use lazy_static::lazy_static;
use std::sync::Mutex;
use crate::dbms::Connection;

pub trait Driver {
    fn name(&self) -> &'static str;

    /// Create initial migration in the folder
    fn create_initial_migration(&self, folder: &PathDir) -> Result<(), Error>;

    /// Opens a new connection to RDBMS and returns it
    fn open_connection(&self, url: &str) -> Result<Box<Connection>, Error>;
}

pub trait DriverFactory: Sync + Send {
    fn name(&self) -> &'static str;
    fn new(&self) -> Box<Driver>;
}

lazy_static! {
    static ref DRIVERS: Mutex<Vec<Box<DriverFactory>>> = Mutex::new(vec![]);
}

pub fn register_driver(driver: Box<DriverFactory>) {
    DRIVERS.lock().unwrap().push(driver);
}

pub fn drivers<R, F>(mut f: F) -> Result<Option<R>, Error>
where
    F: FnMut(&Box<DriverFactory>) -> Option<Result<R, Error>>,
{
    let drivers = DRIVERS.lock().unwrap();

    for factory in &drivers[..] {
        if let Some(r) = f(factory) {
            match r {
                Ok(r) => return Ok(Some(r)),
                Err(e) => return Err(e),
            }
        }
    }

    Ok(None)
}

pub fn lookup(name: &str) -> Option<Box<Driver>> {
    drivers(|factory| {
        if factory.name() == name {
            Some(Ok(factory.new()))
        } else {
            None
        }
    }).unwrap_or(None)
}
