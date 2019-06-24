use crate::args::Args;
use crate::MainResult;
use failure::Error;
use path_abs::{PathDir, PathFile};
use warden_core::config::Config;
use warden_core::migration::algo::Algo;
use warden_core::migration::identity::Identity;
use warden_core::migration::meta::Meta;
use warden_core::migration::path::FileOrDir;
use warden_core::migration::seal::SealMeta;

pub fn run(args: &Args, name: &str) -> MainResult {
    let config = args.get_config()?;

    log::debug!("Initialize a new migration at: {:?}", config.migrations);
    let identity = Identity::new(String::from(name));

    log::debug!("Migration identity: {}", identity);
    create(&config, &identity)?;

    Ok(())
}

fn create(config: &Config, id: &Identity) -> Result<Meta, Error> {
    let root = PathDir::create(config.migrations.join(format!("{}", id)))?;
    let meta = PathFile::create(root.join("meta.yml"))?;
    let source = FileOrDir::from(PathDir::create(root.join("sql"))?);
    let target = root.join("migration.sql");
    let seal_meta = SealMeta::build(root.join("seal.yml"), Algo::default());

    let meta = Meta::create(
        Meta::default_yaml_format_version(),
        meta,
        id.clone(),
        seal_meta,
        source,
        target,
    )?;

    Ok(meta)
}
