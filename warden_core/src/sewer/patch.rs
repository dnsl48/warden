pub mod raw_meta;
pub mod meta;

use crate::path;
use path_abs::PathFile;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Patch {
    id: Uuid,
    source: PathFile
}

impl Patch {
    pub fn new(source: PathFile) -> Patch {
        Patch {
            id: path::to_uuid(source.as_path()),
            source
        }
    }

    pub fn get_id(&self) -> &Uuid {
        &self.id
    }

    pub fn get_source(&self) -> &PathFile {
        &self.source
    }
}
