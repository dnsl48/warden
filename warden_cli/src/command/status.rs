use crate::args::Args;
use crate::grid::{Grid, Grid2};
use crate::migration;
use crate::MainResult;

use failure::Error;

use warden_core::config::Config;
use warden_core::dbms::Connection;
use warden_core::path::relpath;

use log;

pub fn run(args: Args) -> MainResult {
    let config = args.get_config()?;

    let mut grid: Grid2 = Grid::default();

    grid.row(["Repo", &relpath(&config.repository)?]);
    grid.row(["Config", &relpath(&config.config_file)?]);
    let connection = database(&mut grid, &config)?;
    migrations(&mut grid, &config, connection.as_ref())?;

    print!("{}", grid.display());

    Ok(())
}

fn database(grid: &mut Grid2, config: &Config) -> Result<Option<Box<Connection>>, Error> {
    let database_url = if let Some(ref database_url) = config.database_url {
        grid.row(["Database URL", &database_url]);
        database_url
    } else {
        grid.row(["Database URL", "undefined"]);
        return Ok(None);
    };

    log::trace!("Connecting to: {}", database_url);

    let conn = match config.get_dbms_connection() {
        Err(msg) => {
            grid.row(["Connection error", &format!("{}", msg)]);
            return Ok(None);
        }
        Ok(connection) => {
            grid.row(["Connection", "working"]);
            connection
        }
    };

    grid.row(["Catalog", conn.get_catalog()]);

    Ok(Some(conn))
}

fn migrations(grid: &mut Grid2, config: &Config, conn: Option<&Box<Connection>>) -> MainResult {
    let migrations = migration::read_migrations(config);

    let last_deployed = if let Some(ref conn) = conn {
        conn.get_last_deployed_migration()?
    } else {
        None
    };

    grid.row(["Migrations:", "============="]);

    for m in migrations {
        grid.row([
            format!("{}{}", if m.is_sealed() { "  " } else { "* " }, m.get_id()),
            if conn.is_some() {
                let is_deployed = if let Some(last_deployed) = last_deployed {
                    if let Some(id) = m.get_id().get_id() {
                        last_deployed >= id
                    } else {
                        false
                    }
                } else {
                    false
                };

                format!("[{}]", if is_deployed { "+" } else { " " })
            } else {
                String::from("[?]")
            },
        ]);
    }

    Ok(())
}
