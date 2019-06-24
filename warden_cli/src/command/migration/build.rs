use crate::args::Args;
use crate::MainResult;
use path_abs::{PathAbs, PathFile};
use warden_core::migration::{self, meta::Meta};
use warden_core::sewer::Sewer;

pub fn run(args: &Args, pattern: &Option<String>, force: bool) -> MainResult {
    let config = args.get_config()?;
    let meta = migration::fs::lookup(&config, pattern)?;

    log::info!("Found migration: {}", meta.get_identity());

    let target = meta.get_target().absolute()?;

    println!("Found migration: {}", meta.get_identity());

    if !force {
        if let Ok(_) = PathFile::new(&target) {
            Err(failure::err_msg("The migration has already been built"))?
        }
    }

    build(meta.clone(), target)
}

pub fn build(meta: Meta, target: PathAbs) -> MainResult {
    let sewer = Sewer::new(meta)?;
    let graph = sewer.sew_up()?;
    let migration = sewer.sewage(&graph)?;

    Ok(PathFile::create(target)?.write_str(&migration)?)
}
