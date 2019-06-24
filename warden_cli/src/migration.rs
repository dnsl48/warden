use failure::{self, Error};
use path_abs::{PathDir, PathFile};

use warden_core::config::Config;
use warden_core::migration::base36;

use warden_core::migration::{self, identity::Identity, meta::Meta};

pub struct MigrationStatus {
    meta: Meta,
    sealed: bool,
}

impl MigrationStatus {
    pub fn get_id(&self) -> &Identity {
        self.meta.get_identity()
    }

    pub fn is_sealed(&self) -> bool {
        self.sealed
    }

    pub fn from_dir(dir: &PathDir) -> Result<MigrationStatus, Error> {
        let meta_path = dir.join("meta.yml");
        let meta = if let Ok(file) = PathFile::new(&meta_path) {
            Meta::open(file)?
        } else {
            return Err(failure::err_msg(format!(
                "Could not find meta.yml at {}",
                meta_path.to_string_lossy()
            )));
        };

        if base36::decode(meta.get_identity().get_uid()).is_some() {
            Ok(MigrationStatus {
                sealed: PathFile::new(meta.get_seal_meta().get_file()).is_ok(),
                meta,
            })
        } else {
            Err(failure::err_msg(format!(
                "could not decode migration ID ({})",
                meta.get_identity().get_uid()
            )))
        }
    }
}

pub fn read_migrations(config: &Config) -> Vec<MigrationStatus> {
    let mut result = Vec::new();

    migration::fs::foreach_migration_sorted::<_, ()>(&config.migrations, |dir| {
        let dir = match PathDir::new(dir.into_path()) {
            Ok(d) => d,
            _ => return None,
        };

        match MigrationStatus::from_dir(&dir) {
            Ok(status) => result.push(status),
            Err(_) => return None,
        }

        None
    });

    result
}
