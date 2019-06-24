use crate::config::Config;
use crate::migration::identity::Identity;
use crate::migration::meta::Meta;
use failure::{self, Error};
use log;
use path_abs::{PathDir, PathFile};
use walkdir::{DirEntry, WalkDir};

pub fn lookup(config: &Config, pattern: &Option<String>) -> Result<Meta, Error> {
    if let Some(pattern) = pattern {
        log::info!("Looking up pattern: \"{}\"", pattern);

        // TODO: only search among unsigned ones
        if let Some(id) = lookup_pattern(&config.migrations, pattern) {
            Ok(Meta::open(PathFile::new(
                config.migrations.join(format!("{}", id)).join("meta.yml"),
            )?)?)
        } else {
            Err(failure::err_msg(format!(
                "could not determine the migration with pattern \"{}\"",
                pattern
            )))
        }
    } else {
        log::info!("Looking for a single unsigned migration");

        let mut ids = lookup_unsigned(&config.migrations);
        let ids_len = ids.len();

        if ids_len > 1 {
            let mut msg = String::from("You have several unsigned migrations:");
            for id in ids {
                msg.push_str(&format!("\n - {}", id));
            }
            Err(failure::err_msg(msg))
        } else if let Some(id) = ids.pop() {
            Ok(Meta::open(PathFile::new(
                config.migrations.join(format!("{}", id)).join("meta.yml"),
            )?)?)
        } else {
            Err(failure::err_msg("Could not find unsigned migrations"))
        }
    }
}

pub fn lookup_pattern(folder: &PathDir, pattern: &str) -> Option<Identity> {
    let mut result: Option<Identity> = None;

    for entry in WalkDir::new(folder.as_path())
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_entry(|e| e.file_type().is_dir())
        .filter_map(|e| e.ok())
    {
        let file_name = entry.file_name().to_str()?;

        log::trace!("Checking \"{:?}\"", file_name);

        if file_name.starts_with(pattern) && file_name.split_at(pattern.len()).1.starts_with("--") {
            return Identity::from_str(file_name);
        }

        if let Some(ix) = file_name.find("--") {
            if file_name.split_at(ix + 2).1.eq(pattern) {
                if result.is_some() {
                    return None;
                } else {
                    result = Identity::from_str(file_name);
                }
            }
        }
    }

    result
}

pub fn lookup_unsigned(folder: &PathDir) -> Vec<Identity> {
    // more than 1 unsigned is migration is weird,
    // but let's have 8 slots just in case...
    let mut result: Vec<Identity> = Vec::with_capacity(8);

    for entry in WalkDir::new(folder.as_path())
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_entry(|e| e.file_type().is_dir())
        .filter_map(|e| e.ok())
    {
        log::trace!("Checking entry: {:?}", entry);

        let file_name = entry.path();

        if let Ok(meta) = PathFile::new(file_name.join("meta.yml")) {
            let meta = match Meta::open(meta) {
                Ok(meta) => meta,
                _ => continue,
            };

            match PathFile::new(meta.get_seal_meta().get_file()) {
                Err(_) => result.push(meta.get_identity().clone()),
                Ok(_) => (),
            };
        }
    }

    result
}

pub fn foreach_migration<F, R>(folder: &PathDir, mut callback: F) -> Option<R>
where
    F: FnMut(DirEntry) -> Option<R>,
{
    for entry in WalkDir::new(folder.as_path())
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_entry(|e| e.file_type().is_dir())
        .filter_map(|e| e.ok())
    {
        if let Some(r) = callback(entry) {
            return Some(r);
        }
    }

    None
}

pub fn foreach_migration_sorted<F, R>(folder: &PathDir, mut callback: F) -> Option<R>
where
    F: FnMut(DirEntry) -> Option<R>,
{
    for entry in WalkDir::new(folder.as_path())
        .min_depth(1)
        .max_depth(1)
        .sort_by(|a, b| a.file_name().cmp(b.file_name()))
        .into_iter()
        .filter_entry(|e| e.file_type().is_dir())
        .filter_map(|e| e.ok())
    {
        if let Some(r) = callback(entry) {
            return Some(r);
        }
    }

    None
}
