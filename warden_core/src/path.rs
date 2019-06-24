use dirs;
use failure::{self, Error};
use path_abs::{PathArc, PathAbs, PathDir};
use std::iter::FromIterator;
use std::ffi::OsStr;
use std::path::{self, Path};
use uuid::Uuid;

pub fn from_str(path: &str) -> Result<PathArc, Error> {
    let path = if let Some(at) = path.rfind('~') {
        path.split_at(at).1
    } else {
        path
    };

    expand_home(Path::new(path))
}

pub fn normalise(path: &Path) -> Result<PathAbs, Error> {
    Ok(expand_home(path)?.absolute()?)
}

fn expand_home(path: &Path) -> Result<PathArc, Error> {
    if path.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            Ok(PathArc::new(home).join(path.strip_prefix("~")?))
        } else {
            Err(failure::err_msg("Could not determine home dir"))
        }
    } else {
        Ok(PathArc::new(path))
    }
}

pub fn relpath(path: &PathArc) -> Result<String, Error> {
    Ok(relpath_to_base(&PathDir::current_dir()?, path))
}

pub fn relpath_to_base(base: &PathDir, path: &PathArc) -> String {
    let matching = {
        let mut base_compos = base.components();
        let mut path_compos = path.components();
        let mut matching = 0;

        loop {
            if let Some(ref base_item) = base_compos.next() {
                if let Some(ref path_item) = path_compos.next() {
                    if base_item == path_item {
                        matching += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        };

        matching
    };

    if matching == 0 {
        return format!("{}", path.as_path().display());
    }

    let base_compos = base.components().skip(matching);
    let path_compos = path.components().skip(matching);

    let result = path::PathBuf::from_iter(
        base_compos
            .map(|_| path::Component::ParentDir)
            .chain(path_compos)
    );

    format!("{}", result.as_path().display())
}


pub fn to_uuid(source: &Path) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_OID, &format!("{:?}", source).as_bytes()[..])
}


pub fn folder_name(dir: &PathDir) -> Result<&str, Error> {
    match dir.file_name() {
        None => Err(failure::err_msg(format!("Path does not have folder name: {}", dir.as_path().to_string_lossy()))),
        Some(name) => {
            match name.to_str() {
                None => Err(failure::err_msg(format!("Could not read the folder name: {}", name.to_string_lossy()))),
                Some(name) => Ok(name)
            }
        }
    }
}


pub fn os_string(os_str: &OsStr) -> Result<String, Error> {
    os_str.to_os_string().into_string()
    .or_else(|string| Err(
        failure::err_msg(
            format!(
                "Could not convert \"{:?}\" into string",
                string
            )
        )
    ))
}

pub fn os_str(os_str: &OsStr) -> Result<&str, Error> {
    os_str.to_str()
    .ok_or_else(||
        failure::err_msg(
            format!(
                "Could not convert \"{:?}\" into string",
                os_str
            )
        )
    )
}
