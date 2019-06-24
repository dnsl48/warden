use crate::args::Args;
use crate::MainResult;
use path_abs::PathFile;
use warden_core::migration;
use warden_core::path::relpath;

pub fn run(args: &Args, pattern: &Option<String>, mut skip_rebuild: bool) -> MainResult {
    let config = args.get_config()?;
    let meta = migration::fs::lookup(&config, pattern)?;

    log::info!("Found migration: {}", meta.get_identity());

    let seal_meta = meta.get_seal_meta();

    if let Ok(_) = PathFile::new(seal_meta.get_file()) {
        Err(failure::err_msg(format!(
            "\"{}\" has already been sealed {:?}",
            meta.get_identity(),
            relpath(seal_meta.get_file())?
        )))?
    }

    let target = meta.get_target().absolute()?;

    if skip_rebuild {
        if let Ok(_) = PathFile::new(&target) {
            log::warn!("Migration exists, skipping rebuild");
        } else {
            skip_rebuild = false;
        }
    }

    if !skip_rebuild {
        super::build::build(meta.clone(), target.clone())?;
        // let builder = migration::builder::Builder::new(&meta)?;
        // PathFile::create(&target)?.write_str(&builder.generate_migration()?)?;
    }

    let migration = PathFile::new(&target)?;

    Ok(seal_meta.make(&migration.read_string()?.as_bytes())?)
}
