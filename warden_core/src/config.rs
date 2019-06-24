pub mod generator;

use crate::dbms::{self, Connection, Driver};
use crate::path;

use failure::{self, Error};

use path_abs::{PathArc, PathDir, PathFile};
use std::env;

use yamlette::yamlette;
use yamlette::model::Fraction;
use yamlette::model::schema::yamlette::Yamlette;


pub struct Config {
    pub config_file: PathFile,
    pub database_url: Option<String>,
    pub repository: PathDir,
    pub migrations: PathDir,
    pub driver: Box<Driver>
}

impl Config {
    pub fn new(path: &Option<PathArc>) -> Result<Config, Error> {
        let file = Self::find_config_file(path, PathDir::current_dir().ok())?;
        Self::open(file)
    }

    pub fn get_dbms_connection(&self) -> Result<Box<Connection>, Error> {
        let db_url = if let Some(ref db_url) = self.database_url {
            db_url
        } else {
            return Err(failure::err_msg(
                format!(
                    "Database connection URL is undefined (in {})",
                    path::relpath(&self.config_file)?
                )
            ))
        };

        self.driver.open_connection(db_url)
    }

    // pub fn at_dir(args: &Args, cur_dir: PathDir) -> Result<Config, Error> {
    //     let file = Self::find_config_file(args, Some(cur_dir))?;
    //     Self::open(file)
    // }

    pub fn open(file: PathFile) -> Result<Config, Error> { Self::parse_config(file) }

    fn parse_config_v_0_1(file: PathFile) -> Result<Config, Error> {
        let mut file_read = file.read()?;
        let schema = Yamlette::new();
        yamlette!(
            read ;
            &mut file_read ;
            [[], [{
                "connection" => (connection_url:String),
                "repository" => (repo_relpath:String),
                "migrations" => (migrations:String),
                "driver" => (driver:String)
            }]] ;
            { schema: schema }
        );

        let database_url = connection_url;

        let driver = if let Some(driver) = driver {
            if let Some(driver) = dbms::driver::lookup(&driver) {
                driver
            } else {
                Err(failure::err_msg(format!("unknown driver {}", driver)))?
            }
        } else {
            Err(failure::err_msg("driver is not defined"))?
        };

        let cfg_folder = if let Some(p) = file.parent_dir() {
            p
        } else {
            Err(failure::err_msg("could not identify configuration folder"))?
        };

        let repository = if let Some(path) = repo_relpath {
            cfg_folder.join(path).canonicalize()?.into_dir()?
        } else {
            Err(failure::err_msg("repository path is not defined"))?
        };

        let migrations = if let Some(path) = migrations {
            cfg_folder.join(path).canonicalize()?.into_dir()?
        } else {
            Err(failure::err_msg("migrations path is not defined"))?
        };

        Ok(Config {
            config_file: file,
            repository: repository,
            database_url: database_url,
            migrations: migrations,
            driver: driver
        })
    }

    fn parse_config(file: PathFile) -> Result<Config, Error> {
        let mut file_read = file.read()?;
        let schema = Yamlette::new();

        yamlette!(
            read ;
            &mut file_read ;
            [[{
                "version" => (version:Fraction)
            }]] ;
            { schema: schema }
        );

        if let Some(version) = version {
            if version == Fraction::new(1u8, 10u8) {
                Self::parse_config_v_0_1(file)
            } else {
                log::error!("Unsupported config version {:.8}", version);
                Err(failure::err_msg("Unsupported version"))?
            }
        } else {
            log::error!("Config version is not defined");
            Err(failure::err_msg("Unsupported version"))?
        }
    }

    fn find_config_file(path: &Option<PathArc>, cur_dir: Option<PathDir>) -> Result<PathFile, Error> {
        if let Some(ref path) = path {
            Ok(PathFile::new(path.absolute()?)?)
        } else if let Ok(path) = env::var("WARDEN_CONFIG_FILE") {
            Ok(PathFile::new(PathArc::new(path).absolute()?)?)
        } else if let Some (file) = cur_dir.and_then(Self::lookup_warden_config) {
            Ok(file)
        } else {
            Err(failure::err_msg("Configuration file is not defined"))
        }
    }

    /// Look for the .warden/config.yml recursively up the folder tree
    fn lookup_warden_config(dir: PathDir) -> Option<PathFile> {
        if let Some(file) = PathFile::new(dir.join(".warden/config.yml")).ok() {
            log::debug!("Found config: {:?}", file);
            Some(file)
        } else if let Some(parent) = dir.parent_dir() {
            Self::lookup_warden_config(parent)
        } else {
            None
        }
    }
}
