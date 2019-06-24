/*
mod patch;

use crate::migration::meta::Meta;
use crate::path;
use crate::time;
use path_abs::{PathAbs, PathDir, PathFile, PathType};
use failure::Error;
use std::collections::{HashMap, HashSet, LinkedList};

use yamlette::yamlette;
use yamlette::model::Fraction;
use yamlette::model::schema::yamlette::Yamlette;
use yamlette::model::style::ISSUE_TAG;

type PatchId = String;
type PatchMap = HashMap<PatchId, Patch>;
type PatchGraph = LinkedList<PatchId>;

#[derive(Debug)]
pub struct Patch {
    id: PatchId,
    source: PathFile,
    requirements: Vec<PathFile>
}

impl Patch {
    pub fn id(&self) -> &PatchId {
        &self.id
    }

    pub fn new(src: PathFile) -> Result<Patch, Error> {
        let id = Self::get_id_for(&src);
        let requirements = Self::read_requirements(&src)?;

        Ok(Patch {id: id, source: src, requirements: requirements})
    }

    fn read_requirements(src: &PathFile) -> Result<Vec<PathFile>, Error> {
        patch::requirements(src)
    }

    pub fn get_source(&self) -> &PathFile {
        &self.source
    }

    pub fn get_id_for(path: &PathAbs) -> PatchId {
        format!("{:?}", path)
    }

    pub fn add_requirement(&mut self, id: PathFile) {
        self.requirements.push(id)
    }

    pub fn get_requirements(&self) -> &[PathFile] {
        &self.requirements
    }
}

pub struct Builder {
    meta: Meta,
    graph: PatchGraph,
    map: PatchMap
}

impl Builder {
    pub fn generate_migration(&self) -> Result<String, Error> {
        let mut result = String::new();
        result.push_str("-- ");
        result.push_str(&self.yamlette()?.trim().replace("\n", "\n-- "));
        result.push_str("\n\n");
        result.push_str(&self.generate_sql()?.trim());

        Ok(result)
    }

    fn generate_sql(&self) -> Result<String, Error> {
        let mut result = String::new();
        let parent_path = self.meta.get_path().parent_dir().ok_or(failure::err_msg("Unexpected metadata path"))?;

        for patch_id in &self.graph {
            let relpath = path::relpath_to_base(&parent_path, self.map[patch_id].get_source())?;

            result.push_str(&format!("-- BEGIN: {}\n\n", relpath));
            result.push_str(&self.map[patch_id].get_source().read_string()?.trim());
            result.push_str(&format!("\n\n-- END: {}\n\n", relpath));
            result.push_str("\n\n")
        }

        Ok(result)
    }

    fn yamlette(&self) -> Result<String, Error> {
        let version = Fraction::new(1u8, 10u8);
        let schema = Yamlette::new();

        let ts = time::yamlette_timestamp_value();

        let mut patches = Vec::new();
        let parent_path = self.meta.get_path().parent_dir().ok_or(failure::err_msg("Unexpected metadata path"))?;

        for patch_id in &self.graph {
            let relpath = path::relpath_to_base(&parent_path, self.map[patch_id].get_source())?;
            patches.push(relpath);
        }

        Ok(yamlette!(
            write ;
            [
                [ % BORDER_TOP => { "version": version } ],
                [ % BORDER_BOT => {
                    "timestamp": (# ISSUE_TAG => ts),
                    "manifest": patches
                } ]
            ]
            ; { schema: schema }
        )?)
    }

    pub fn new(meta: &Meta) -> Result<Builder, Error> {
        let (map, graph) = meta.get_source().map(Self::from_file, Self::from_dir)?;
        let builder = Builder {
            meta: meta.clone(),
            graph: graph,
            map: map
        };

        Ok(builder)
    }

    fn validate_graph(path: &PathDir, map: &PatchMap, graph: &PatchGraph) -> Result<(), Error> {
        let mut passed = HashSet::new();
        for patch_id in graph {
            Self::validate_patch(path, patch_id, map, &passed)?;
            let source = map.get(patch_id).ok_or(failure::err_msg(format!("Inconsistent patch map, does not have id {:?}", patch_id)))?.source;
            passed.insert(&source);
        }

        Ok(())
    }

    fn validate_patch(path: &PathDir, patch_id: &PatchId, map: &PatchMap, passed: &HashSet<&PathFile>) -> Result<(), Error> {
        let patch = map.get(patch_id).ok_or(failure::err_msg(format!("Inconsistent patch map, does not have id {:?}", patch_id)))?;

        for req in patch.get_requirements() {
            if !passed.contains(&req) {
                let parent_path = path.parent_dir().ok_or(failure::err_msg("Unexpected metadata path"))?;
                let patch = path::relpath_to_base(&parent_path, &path.join(patch_id))?;
                Err(failure::err_msg(format!("patch {:?} requires {:?}, but it's not there", patch, req)))?;
            }
        }

        Ok(())
    }

    fn from_file(patch: &PathFile) -> Result<(PatchMap, PatchGraph), Error> {
        let patch = Patch::new(patch.clone())?;

        let mut graph = LinkedList::new();
        graph.push_back(patch.id().clone());

        let mut map = HashMap::with_capacity(1);
        map.insert(patch.id().clone(), patch);

        Ok((map, graph))
    }

    fn from_dir(patch: &PathDir) -> Result<(PatchMap, PatchGraph), Error> {
        let mut map = HashMap::new();
        let graph = Self::new_graph(patch, &mut map, &[])?;

        Self::validate_graph(patch, &map, &graph)?;

        Ok((map, graph))
    }

    fn new_graph(path: &PathDir, map: &mut PatchMap, requirements: &[&PathFile]) -> Result<PatchGraph, Error> {
        let mut graph = LinkedList::new();
        let (files, dirs) = Self::slice_folder(path)?;

        // map of file names to patch ids
        let mut file_names: HashMap<String, PatchId> = HashMap::new();

        for file in files {
            let mut patch = Patch::new(file)?;

            for req in requirements {
                patch.add_requirement((*req).clone());
            }

            {
                let src = patch.get_source();
                let path = src.as_path();
                let mut file_name: String = src.as_path()
                    .file_name().expect(&format!("Could not extract file name of \"{:?}\"", path))
                    .to_str().expect(&format!("UTF-8 conversion failed for \"{:?}\"", path)).into();

                if file_name.ends_with(".sql") {
                    file_name.truncate(file_name.len() - 4);
                    file_names.insert(file_name, patch.id().clone());
                } else {
                    continue;
                }
            }

            graph.push_back(patch.id().clone());
            map.insert(patch.id().clone(), patch);
        }

        for folder in dirs {
            let mut requirements = Vec::new();

            {
                let path = folder.as_path();
                let file_name = folder.as_path()
                    .file_name().expect(&format!("Could not extract name of \"{:?}\"", path))
                    .to_str().expect(&format!("UTF-8 conversion failed for \"{:?}\"", path));

                if let Some(id) = file_names.get(file_name) {
                    requirements.push(id);
                }
            }

            graph.append(&mut Self::new_graph(&folder, map, &requirements[..])?);
        }

        Ok(graph)
    }

    fn slice_folder(path: &PathDir) -> Result<(Vec<PathFile>, Vec<PathDir>), Error> {
        let mut files = Vec::new();
        let mut dirs = Vec::new();

        for item in path.read_dir()?.filter_map(|e| e.ok()) {
            match item {
                 PathType::File(file) => files.push(file),
                 PathType::Dir(dir) => dirs.push(dir)
            };
        }

        Ok((files, dirs))
    }
}
*/
