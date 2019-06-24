use failure::Error;
use crate::migration::meta::Meta;

pub trait Connection {
    fn get_catalog(&self) -> &str;

    fn get_last_deployed_migration(&self) -> Result<Option<u128>, Error>;

    fn deploy(&self, meta: Meta) -> Result<(), Error>;
}
