//! Map of the migration containing all the metadata

use crate::migration::meta::Meta as MigrationMeta;
use failure::Error;

use super::patch::{meta::Meta as PatchMeta, Patch};
use super::raw_map::RawMap;
use crate::path;
use std::collections::HashMap;
use uuid::Uuid;

/// This map contains processed and validated patches
#[derive(Clone, Debug)]
pub struct Map {
    patches: HashMap<Uuid, (Patch, PatchMeta)>,
}

impl Map {
    pub fn get_patches(&self) -> &HashMap<Uuid, (Patch, PatchMeta)> {
        &self.patches
    }

    pub fn get_patch(&self, key: &Uuid) -> Option<&(Patch, PatchMeta)> {
        self.patches.get(key)
    }

    pub fn from_raw(migration_meta: &MigrationMeta, raw: &RawMap) -> Result<Self, Error> {
        log::trace!("Map::from_raw | remapping the patches");
        let mut patches = HashMap::with_capacity(raw.get_patches().len());

        for (key, (ref patch, ref patch_meta)) in raw.get_patches() {
            log::debug!(
                "Reading patch {} ({})",
                migration_meta.get_source().map(
                    |_| path::printable(patch.get_source()),
                    |d| path::printable_rel_to_base(d, patch.get_source())
                ),
                format!("{:.64}", patch_meta.get_weight())
            );
            let patch_meta = PatchMeta::from_raw(migration_meta, raw, patch, patch_meta)?;
            patches.insert(*key, (patch.clone(), patch_meta));
        }

        Ok(Map { patches: patches })
    }
}
