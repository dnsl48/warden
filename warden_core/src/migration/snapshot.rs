//! A snapshot is a full migration folder taken as an archive (binary string)

use super::meta::Meta;
use failure::Error;
use libflate::gzip::Encoder;
use std::fmt::{self, Display, Formatter};
use tar;

/// Possible archive formats of Snapshot
#[derive(Copy, Clone, Debug)]
pub enum Format {
    /// ".tar.gz"
    TarGz,
}

impl Display for Format {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Format {
    pub fn as_str(&self) -> &str {
        match *self {
            Format::TarGz => ".tar.gz",
        }
    }

    pub fn from(value: &str) -> Result<Format, Error> {
        match value {
            ".tar.gz" => Ok(Format::TarGz),
            _ => Err(failure::err_msg(format!(
                "Unknown snapshot format: {}",
                value
            ))),
        }
    }
}

/// Snapshot is a migration folder archived
// #[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    /// Format of the archive
    pub format: Format,

    /// Binary representation of the archive
    pub data: Vec<u8>,
}

impl Snapshot {
    pub fn take(format: Format, meta: &Meta) -> Result<Snapshot, Error> {
        let data = match format {
            Format::TarGz => Self::tar_gz(&meta)?,
        };

        Ok(Self { format, data })
    }

    fn tar_gz(meta: &Meta) -> Result<Vec<u8>, Error> {
        let mut enc = Encoder::new(Vec::new())?;

        {
            let src = meta.get_base();
            let mut arch = tar::Builder::new(&mut enc);

            arch.append_dir_all(
                src.as_path()
                    .file_name()
                    .ok_or(failure::err_msg("Nameless migration..."))?,
                src.absolute()?,
            )?;

            arch.finish()?;
        }

        Ok(enc.finish().into_result()?)
    }
}
