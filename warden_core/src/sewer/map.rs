//! Map of the migration containing all the metadata

use crate::migration::meta::Meta as MigrationMeta;
use failure::Error;

use super::patch::{meta::Meta as PatchMeta, Patch};
use super::raw_map::RawMap;
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
        let mut patches = HashMap::with_capacity(raw.get_patches().len());

        for (key, (ref patch, ref patch_meta)) in raw.get_patches() {
            let patch_meta = PatchMeta::from_raw(migration_meta, raw, patch, patch_meta)?;
            patches.insert(*key, (patch.clone(), patch_meta));
        }

        Ok(Map { patches: patches })
    }
}
