use warden_core::config::{self, Config};
use warden_core::path::relpath;
use failure::Error;
use log;
use path_abs::{PathFile, PathDir};

pub fn init(driver: &str, root: PathDir) -> Result<Config, Error> {
    let folder = root.join(".warden").join(config::generator::migrations_relpath().into_owned()).absolute()?;

    log::debug!("Init migrations folder: {:?}", relpath(&folder)?);
    PathDir::create(folder)?;
    let config = config(driver, root)?;

    log::debug!("Creating initial migration for {}", driver);
    config.driver.create_initial_migration(&config.migrations)?;

    Ok(config)
}

/// Initialise config
pub fn config(driver: &str, root: PathDir) -> Result<Config, Error> {
    // log::debug!("Reading config: {:?}", root.join(".warden"));
    let config_path = PathDir::create(root.join(".warden"))?;
    // log::debug!("Read config: {:?}", config_path);

    let file = {
        let path = config_path.join("config.yml").absolute()?;
        if let Ok(path) = PathFile::from_abs(path.clone()) {
            log::warn!("The config already exists: {:?}", relpath(&path)?);
            return Ok(Config::open(path)?)
        };
        path
    };

    log::debug!("Config file: {:?}", file);

    let cfg = config::generator::generate_initial(driver)?;
    let file = PathFile::create(file)?;
    file.write_str(&cfg)?;

    Ok(Config::open(file)?)
}
