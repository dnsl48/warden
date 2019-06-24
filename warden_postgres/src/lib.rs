mod api;
pub mod driver;
pub mod connection;

pub fn zero_migration_tar() -> &'static [u8] {
    let _archive = include_bytes!(concat!(env!("OUT_DIR"), "/000000--warden-init.tar.gz"));

    &_archive[..]
}
