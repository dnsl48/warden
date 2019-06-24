use crate::args::Args;
use crate::grid::{Grid, Grid3};
use crate::MainResult;
use path_abs::PathFile;
use warden_core::migration::{self, identity::Identity, meta::Meta};
use warden_core::path;

pub fn run(args: &Args) -> MainResult {
    let config = args.get_config()?;

    let mut grid: Grid3 = Grid::default();

    migration::fs::foreach_migration_sorted::<_, ()>(&config.migrations, |dir| {
        log::trace!("Checking folder: {:?}", dir);

        let id = if let Some(id) = Identity::from_str(dir.file_name().to_str()?) {
            id
        } else {
            log::trace!("Could not parse Identity. Skip...");
            return None;
        };

        let meta = if let Ok(file) =
            PathFile::new(config.migrations.join(format!("{}", &id)).join("meta.yml"))
        {
            file
        } else {
            log::trace!("Could not FIND meta.yml. Skip...");
            return None;
        };

        let meta = match Meta::open(meta) {
            Ok(meta) => meta,
            Err(error) => {
                let meta_file =
                    PathFile::new(config.migrations.join(format!("{}", &id)).join("meta.yml"))
                        .unwrap();
                log::warn!(
                    "Reading {} error: {}",
                    path::relpath(&meta_file).unwrap_or_else(|_| format!("{:?}", meta_file)),
                    error
                );
                log::trace!("Could not READ meta.yml. Skip...");

                return None;
            }
        };

        let sealed = PathFile::new(meta.get_seal_meta().get_file()).is_ok();
        let deployed = false;

        grid.row([
            format!("{}", if sealed { "" } else { "*" }),
            format!("{}", id),
            format!("[{}]", if deployed { "x" } else { " " }),
        ]);

        None
    });

    println!("");
    println!("{}", grid.display());

    Ok(())
}
