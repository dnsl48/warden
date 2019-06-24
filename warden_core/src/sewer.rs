//! the module for sewing up the patches into a migration

pub mod map;
pub mod patch;
pub mod raw_map;

use crate::migration::meta::Meta;
use crate::path;
use crate::time;
use failure::Error;
use map::Map;
use patch::{meta::Meta as PatchMeta, Patch};
use path_abs::{PathArc, PathDir, PathFile};
use raw_map::RawMap;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use uuid::Uuid;

use yamlette::model::schema::yamlette::Yamlette;
use yamlette::model::style::ISSUE_TAG;
use yamlette::model::Fraction;
use yamlette::yamlette;

/// Sewer parses patches and builds a single SQL file with the migration
#[derive(Clone, Debug)]
pub struct Sewer {
    meta: Meta,
    map: Map,
}

impl Sewer {
    pub fn new(meta: Meta) -> Result<Sewer, Error> {
        let raw_map = RawMap::new(&meta)?;
        let source_base = meta.get_source_base();

        // add extra dependencies (e.g. parent nodes)
        let raw_map = Self::raw_map_update(&source_base, raw_map)?;

        let map = Map::from_raw(&meta, &raw_map)?;

        Ok(Sewer { map: map, meta })
    }

    /// Build the actual migration, combining all the patches
    /// This method usually takes result of `sew_up` as its argument
    pub fn sewage(&self, migration: &[Uuid]) -> Result<String, Error> {
        Ok(format!(
            "-- {}\n\n{}",
            &self.yamlette(migration)?.trim().replace("\n", "\n-- "),
            &self.merge_up(migration)?.trim()
        ))
    }

    fn yamlette(&self, migration: &[Uuid]) -> Result<String, Error> {
        let version = Fraction::new(1u8, 10u8);
        let schema = Yamlette::new();

        let ts = time::yamlette_timestamp_value();

        let mut manifest = Vec::new();
        let base_path = self
            .meta
            .get_path()
            .parent_dir()
            .ok_or(failure::err_msg("Unexpected metadata path"))?;

        for id in migration {
            let path = path::relpath_to_base(&base_path, self.map.get_patches()[id].0.get_source());
            manifest.push(path);
        }

        Ok(yamlette!(
            write ;
            [
                [ % BORDER_TOP => { "version": version } ],
                [ % BORDER_BOT => {
                    "timestamp": (# ISSUE_TAG => ts),
                    "manifest": manifest
                } ]
            ]
            ; { schema: schema }
        )?)
    }

    fn merge_up(&self, patches: &[Uuid]) -> Result<String, Error> {
        let mut result_len = 0;

        for id in patches {
            let source = self.map.get_patches()[id].0.get_source();
            let len = source.read()?.metadata()?.len();

            result_len += len;

            result_len += 1000; // "-- BEGIN: {}\n\n"
            result_len += 1000; // "\n\n-- END: {}\n\n"
        }

        let mut merge = String::with_capacity(result_len as usize);
        let base_path = self
            .meta
            .get_path()
            .parent_dir()
            .ok_or(failure::err_msg("Unexpected metadata path"))?;

        for id in patches {
            let source = self.map.get_patches()[id].0.get_source();
            let path = path::relpath_to_base(&base_path, &source);

            merge.push_str(&format!("-- BEGIN: {}\n\n", path));
            merge.push_str(source.read_string()?.trim());
            merge.push_str(&format!("\n\n-- END: {}\n\n", path));
        }

        Ok(merge)
    }

    /// Organise the patches and build up their deps
    pub fn sew_up(&self) -> Result<Vec<Uuid>, Error> {
        let patches = self.map.get_patches();

        let mut collection: Vec<&(Patch, PatchMeta)> = patches.iter().map(|(_, v)| v).collect();
        collection.sort_unstable_by(|a, b| a.1.get_weight().partial_cmp(b.1.get_weight()).unwrap());

        let mut awaiting: HashSet<Uuid> = HashSet::with_capacity(patches.len());
        let mut handled: HashSet<Uuid> = HashSet::with_capacity(patches.len());
        let mut migration: Vec<Uuid> = Vec::with_capacity(patches.len());

        for (ref patch, _) in collection {
            Self::patch_up(
                &mut awaiting,
                &mut handled,
                &mut migration,
                patches,
                patch.get_id(),
            )?;
        }

        Ok(migration)
    }

    fn patch_up(
        awaiting: &mut HashSet<Uuid>,
        handled: &mut HashSet<Uuid>,
        migration: &mut Vec<Uuid>,
        patches: &HashMap<Uuid, (Patch, PatchMeta)>,
        key: &Uuid,
    ) -> Result<(), Error> {
        if handled.contains(key) {
            return Ok(());
        }
        if awaiting.contains(key) {
            return Err(Self::looped_recursion_error(awaiting, patches));
        }

        awaiting.insert(*key);
        let (_, ref meta) = patches[&key];
        for req in meta.get_requirements() {
            Self::patch_up(awaiting, handled, migration, patches, req)?;
        }
        awaiting.remove(key);

        migration.push(*key);
        handled.insert(*key);

        Ok(())
    }

    #[inline(never)]
    fn looped_recursion_error(
        awaiting: &HashSet<Uuid>,
        patches: &HashMap<Uuid, (Patch, PatchMeta)>,
    ) -> Error {
        let mut msg = String::from("Looped recursion detected:\n");
        for a_key in awaiting.iter() {
            write!(&mut msg, " - {:?}\n", patches[&a_key].0.get_source()).ok();
        }
        failure::err_msg(msg)
    }

    /// 1. for each patch and add its parent package as a dependency
    fn raw_map_update(source_base: &PathDir, mut map: RawMap) -> Result<RawMap, Error> {
        let patches = map.get_patches_mut();

        for (_, (ref patch, ref mut meta)) in patches.iter_mut() {
            if let Some(pdir) = patch.get_source().parent_dir() {
                let mut parent_file = pdir.as_path().as_os_str().to_os_string();
                parent_file.push(".sql");

                match PathFile::new(&parent_file) {
                    Ok(_) => meta.add_requirement(format!(
                        "/{}",
                        path::relpath_to_base(source_base, &PathArc::new(&parent_file))
                    )),
                    Err(_) => (),
                };
            }
        }

        Ok(map)
    }
}
