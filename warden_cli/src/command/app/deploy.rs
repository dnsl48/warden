use crate::args::Args;
use crate::grid::{Grid, Grid3};
use crate::MainResult;
use path_abs::PathFile;
use warden_core::migration::{self, identity::Identity, meta::Meta};

pub fn run(args: &Args) -> MainResult {
    let config = args.get_config()?;
    let conn = config.get_dbms_connection()?;

    let last_deployed = conn.get_last_deployed_migration()?;
    // let last_deployed_uid = last_deployed.clone().map(base36::encode);

    let mut to_be_deployed = Vec::new();

    migration::fs::foreach_migration_sorted::<_, ()>(&config.migrations, |dir| {
        let identity = if let Some(identity) = Identity::from_str(dir.file_name().to_str()?) {
            identity
        } else {
            return None;
        };

        if let Some(last_deployed) = last_deployed {
            if let Some(id) = identity.get_id() {
                if last_deployed >= id {
                    return None;
                }
            }
        }

        let meta = if let Ok(file) = PathFile::new(
            config
                .migrations
                .join(format!("{}", &identity))
                .join("meta.yml"),
        ) {
            file
        } else {
            return None;
        };

        let meta = if let Ok(meta) = Meta::open(meta) {
            meta
        } else {
            return None;
        };

        let sealed = PathFile::new(meta.get_seal_meta().get_file()).is_ok();

        if !sealed {
            return None;
        }

        to_be_deployed.push(meta);

        None
    });

    let mut grid: Grid3 = Grid::default();

    for meta in to_be_deployed {
        let uid = &format!("{}", meta.get_identity());
        let result = conn.deploy(meta);
        grid.row([" -", uid, if result.is_ok() { "[x]" } else { "[error!]" }]);

        if result.is_err() {
            print!("{}", grid.display());
            result?;
        }
    }

    print!("{}", grid.display());
    Ok(())
}
