use either::{Either, Left, Right};
use path_abs::{PathAbs, PathDir, PathFile};
use failure::Error;

// pub use crate::path::*;

#[derive(Debug, Clone)]
pub struct FileOrDir {
    path: Either<PathFile, PathDir>
}

impl FileOrDir {
    pub fn map<F, D, R>(&self, mut file_mapper: F, mut dir_mapper: D) -> Result<R, Error>
    where
        F: FnMut(&PathFile) -> Result<R, Error>,
        D: FnMut(&PathDir) -> Result<R, Error>
    {
        match self.path {
            Left(ref file) => file_mapper(file),
            Right(ref dir) => dir_mapper(dir)
        }
    }

    pub fn get_path(&self) -> PathAbs {
        match self.path {
            Left(ref file) => {
                let mut p = PathAbs::mock("/");
                p.clone_from(file);
                p
            }
            Right(ref dir) => {
                let mut p = PathAbs::mock("/");
                p.clone_from(dir);
                p
            }
        }
    }
}


impl From<PathFile> for FileOrDir {
    fn from(file: PathFile) -> FileOrDir {
        FileOrDir {
            path: Left(file)
        }
    }
}


impl From<PathDir> for FileOrDir {
    fn from(dir: PathDir) -> FileOrDir {
        FileOrDir {
            path: Right(dir)
        }
    }
}
