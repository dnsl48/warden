use chrono::prelude::{DateTime, FixedOffset};
use failure::Error;
use postgres::transaction::Transaction;

pub fn do_register_migration(
    connection: &Transaction,
    id: u128,
    name: &str,
    source: &str,
    snapshot_format: &str,
    snapshot_data: &[u8],
    seal_generated_at: &DateTime<FixedOffset>,
    seal_algo: &str,
    seal_data: &[u8],
) -> Result<(), Error> {
    Ok(connection
        .execute(
            "select warden.do_register_migration(($1::text)::int8, $2, $3, $4, $5, $6, $7, $8)",
            &[
                // "postgres=0.15.2" does not support u128 yet, so we pass it as a string
                &id.to_string(),
                &name,
                &source,
                &snapshot_format,
                &snapshot_data,
                seal_generated_at,
                &seal_algo,
                &seal_data,
            ],
        )
        .map(|_| ())?)
}

pub fn do_deploy_migration(connection: &Transaction, id: u128) -> Result<(), Error> {
    Ok(connection
        .execute(
            "select warden.do_deploy_migration(($1::text)::int8)",
            &[&id.to_string()],
        )
        .map(|_| ())?)
}
