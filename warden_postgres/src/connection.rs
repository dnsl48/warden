use failure::Error;

use crate::api;
use path_abs::FileRead;
use postgres::{self, transaction::Transaction, TlsMode};
use std::ops::Deref;
use warden_core::migration::snapshot;
use warden_core::{dbms, migration::meta::Meta};

#[derive(Debug)]
pub struct Connection {
    connection: postgres::Connection,
    catalog: String,
    initialised: bool,
}

impl dbms::Connection for Connection {
    fn get_catalog(&self) -> &str {
        &self.catalog
    }

    fn get_last_deployed_migration(&self) -> Result<Option<u128>, Error> {
        if !self.is_initialised()? {
            return Ok(None);
        }

        let result: i64 = self
            .connection
            .query("select warden.get_latest_deployed_migration()", &[])?
            .get(0)
            .get_opt(0)
            .ok_or(failure::err_msg("Could not fetch warden metadata"))??;

        Ok(Some(result as u128))
    }

    fn deploy(&self, meta: Meta) -> Result<(), Error> {
        if !self.is_initialised()? {
            let id = meta.get_identity().get_id().unwrap_or(1);

            if id == 0 {
                return self.deploy_initial(meta);
            } else {
                return Err(failure::err_msg(
                    format!(
                        r#"The database is not initialised with Warden. Error trying to deploy migration "{}". The initial migration must be deployed first"#,
                        meta.get_identity()
                    )
                ));
            }
        }

        let transaction = self.connection.transaction()?;
        self.register_migration(&transaction, &meta)?;
        self.deploy_migration(&transaction, &meta)?;
        Ok(transaction.commit()?)
    }
}

impl Connection {
    fn is_initialised(&self) -> Result<bool, Error> {
        Ok(self.initialised || is_initialised(&self.connection, &self.catalog)?)
    }

    fn deploy_migration(&self, transaction: &Transaction, meta: &Meta) -> Result<(), Error> {
        api::do_deploy_migration(
            transaction,
            meta.get_identity()
                .get_id()
                .ok_or(failure::err_msg("could not decode migration id"))?,
        )
    }

    fn register_migration(&self, transaction: &Transaction, meta: &Meta) -> Result<(), Error> {
        let snapshot = snapshot::Snapshot::take(snapshot::Format::TarGz, &meta)?;
        let seal = meta.get_seal_meta().read_the_seal()?;

        api::do_register_migration(
            transaction,
            meta.get_identity()
                .get_id()
                .ok_or(failure::err_msg("could not decode migration id"))?,
            meta.get_identity().get_name(),
            &FileRead::read(meta.get_target())?.read_string()?,
            snapshot.format.as_str(),
            &snapshot.data,
            &seal.timestamp,
            seal.algo.stringify(),
            &seal.sign,
        )
    }

    fn deploy_initial(&self, meta: Meta) -> Result<(), Error> {
        log::trace!("Deploying initial migration");
        let sql = &FileRead::read(meta.get_target())?.read_string()?;
        let transaction = self.connection.transaction()?;
        transaction.batch_execute(sql)?;

        self.register_migration(&transaction, &meta)?;
        transaction.execute(
            "update warden.migration set deploy_ts = now() where id = 0",
            &[],
        )?;

        Ok(transaction.commit()?)
    }
}

impl Deref for Connection {
    type Target = postgres::Connection;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

pub fn open(url: &str) -> Result<Connection, Error> {
    let connection = postgres::Connection::connect(url.clone(), TlsMode::None)?;

    connection.execute(
        "select set_config('application_name', 'warden', false)",
        &[],
    )?;

    let catalog: String = connection
        .query("select current_database()", &[])?
        .get(0)
        .get_opt(0)
        .ok_or(failure::err_msg("Could not fetch current catalog"))??;

    let initialised: bool = connection
        .query(
            "select
              count(*) = 1
            from
              information_schema.tables
            where
              table_catalog = $1
            and
              table_schema = 'warden'
            and
              table_name = 'migration'
            ",
            &[&catalog],
        )?
        .get(0)
        .get_opt(0)
        .ok_or(failure::err_msg("Could not read information schema"))??;

    Ok(Connection {
        connection: connection,
        catalog: catalog,
        initialised: initialised,
    })
}

fn is_initialised(connection: &postgres::Connection, catalog: &str) -> Result<bool, Error> {
    let result: bool = connection
        .query(
            "select
              count(*) = 1
            from
              information_schema.tables
            where
              table_catalog = $1
            and
              table_schema = 'warden'
            and
              table_name = 'migration'
            ",
            &[&catalog],
        )?
        .get(0)
        .get_opt(0)
        .ok_or(failure::err_msg("Could not read information schema"))??;

    Ok(result)
}
