use super::patch::{raw_meta::RawMeta, Patch};
use crate::migration::meta::Meta;
use crate::path;
use failure::Error;
use path_abs::{PathDir, PathFile};
use std::collections::HashMap;
use uuid::Uuid;
use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct RawMap {
    patches: HashMap<Uuid, (Patch, RawMeta)>,
}

impl RawMap {
    pub fn get_patches(&self) -> &HashMap<Uuid, (Patch, RawMeta)> {
        &self.patches
    }

    pub fn get_patches_mut(&mut self) -> &mut HashMap<Uuid, (Patch, RawMeta)> {
        &mut self.patches
    }

    fn empty() -> Self {
        RawMap {
            patches: HashMap::new(),
        }
    }

    pub fn new(meta: &Meta) -> Result<RawMap, Error> {
        meta.get_source().map(
            |file| Self::from_file(meta, file),
            |dir| Self::from_dir(meta, dir),
        )
    }

    fn patch_up(&mut self, tuple: (Patch, RawMeta)) {
        self.patches.insert(*tuple.0.get_id(), tuple);
    }

    fn build_patch(
        migration_meta: &Meta,
        base: &PathDir,
        src: &PathFile,
    ) -> Result<(Patch, RawMeta), Error> {
        let meta = RawMeta::from_file(migration_meta, base, src)?;
        Ok((Patch::new(src.clone()), meta))
    }

    fn from_file(meta: &Meta, file: &PathFile) -> Result<RawMap, Error> {
        let mut map = Self::empty();

        let base = meta.get_base();

        map.patch_up(Self::build_patch(meta, &base, file)?);

        Ok(map)
    }

    fn from_dir(migration_meta: &Meta, dir: &PathDir) -> Result<RawMap, Error> {
        log::trace!(
            "building from \"{}\"",
            path::relpath(dir).unwrap_or_else(|_| format!("{:?}", dir))
        );
        let mut map = Self::empty();

        for entry in WalkDir::new(dir.as_path())
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            if !path::os_str(entry.path().as_os_str())?.ends_with(".sql") {
                continue;
            }

            let file = PathFile::new(entry.path())?;
            map.patch_up(Self::build_patch(migration_meta, &dir, &file)?);
        }

        log::trace!("The map is built");

        Ok(map)
    }
}
