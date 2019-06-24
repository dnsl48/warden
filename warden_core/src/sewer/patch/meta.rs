use super::super::raw_map::RawMap;
use super::raw_meta::RawMeta;
use super::Patch;
use crate::migration::meta::Meta as MigrationMeta;
use crate::path;
use failure::Error;
use fraction::BigFraction;
use path_abs::{PathDir, PathFile};
use uuid::Uuid;

/// Meta contains parsed, processed and validated data
#[derive(Clone, Debug)]
pub struct Meta {
    path: String,
    requirements: Vec<Uuid>,
    weight: BigFraction,
}

impl Meta {
    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn get_requirements(&self) -> &[Uuid] {
        &self.requirements
    }

    pub fn get_weight(&self) -> &BigFraction {
        &self.weight
    }

    pub fn from_raw(
        meta: &MigrationMeta,
        raw_map: &RawMap,
        patch: &Patch,
        raw: &RawMeta,
    ) -> Result<Self, Error> {
        let source_base = meta.get_source_base();
        let requirements = Self::build_requirements(&source_base, patch, raw_map, &raw)?;

        Ok(Meta {
            path: String::from(raw.get_path()),
            requirements,
            weight: raw.get_weight().clone(),
        })
    }

    fn build_requirements(
        source_base: &PathDir,
        patch: &Patch,
        raw_map: &RawMap,
        raw_meta: &RawMeta,
    ) -> Result<Vec<Uuid>, Error> {
        let mut result = Vec::new();
        let patch_base = path::normalise(
            &patch
                .get_source()
                .parent_dir()
                .ok_or_else(|| failure::err_msg("A file without parent..."))?,
        )?;
        let source_base = source_base.absolute()?;

        for req in raw_meta.get_requirements() {
            let req_path = path::from_str(req)?;

            let (base, req_path) = if req_path.has_root() {
                (&source_base, &req[1..])
            } else {
                (&patch_base, &req[..])
            };

            let requirement = PathFile::new(path::normalise(&base.join(req_path))?)?;

            let uuid = path::to_uuid(requirement.as_path());

            if raw_map.get_patches().contains_key(&uuid) {
                result.push(uuid);
            } else {
                return Err(failure::err_msg(format!(
                    r#"Unknown resource "{}" required by "{:?}""#,
                    req,
                    patch.get_source()
                )));
            };
        }

        Ok(result)
    }
}
