use exitfailure::ExitFailure;

use libflate::gzip::Encoder;
use path_abs::{FileRead, FileWrite, PathDir};
use std::{env, io};
use tar;

fn main() -> Result<(), ExitFailure> {
    let dst = PathDir::new(env::var("OUT_DIR").unwrap())?;
    let src = PathDir::new(env::var("CARGO_MANIFEST_DIR").unwrap())?;

    let dst_file = FileWrite::create(dst.join("000000--warden-init.tar"))?;
    let dst_file_path = dst_file.path().clone();
    let mut arch = tar::Builder::new(dst_file);
    arch.append_dir_all(
        "000000--warden-init",
        src.join("db/migrations/000000--warden-init").absolute()?,
    )?;
    arch.finish()?;

    let mut src_file = FileRead::read(dst_file_path)?;
    let dst_file = FileWrite::create(dst.join("000000--warden-init.tar.gz"))?;
    let mut gzipper = Encoder::new(dst_file)?;

    io::copy(&mut src_file, &mut gzipper)?;
    gzipper.finish();

    Ok(())
}
