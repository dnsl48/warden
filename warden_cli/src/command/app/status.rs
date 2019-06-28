use crate::args::Args;
use crate::grid::{Grid, Grid2};
use crate::MainResult;

use colored::*;
use failure::Error;

use warden_core::config::Config;
use warden_core::dbms::Connection;
use warden_core::path::relpath;

use log;

const KEY_COLOUR: &'static str = "green";
const VAL_COLOUR: &'static str = "blue";
const ERR_COLOUR: &'static str = "red";

pub fn run(args: &Args) -> MainResult {
    let config = args.get_config()?;

    let mut grid: Grid2 = Grid::default();

    grid.row(["", ""]);
    grid.row([
        "Repo".color(KEY_COLOUR),
        relpath(&config.repository)?.color(VAL_COLOUR),
    ]);

    grid.row([
        "Config".color(KEY_COLOUR),
        relpath(&config.config_file)?.color(VAL_COLOUR),
    ]);

    let connection = database(&mut grid, &config)?;
    crate::command::migration::list::migrations(&mut grid, &config, connection.as_ref())?;

    grid.row(["", ""]);

    print!("{}", grid.display());

    Ok(())
}

fn database(grid: &mut Grid2, config: &Config) -> Result<Option<Box<Connection>>, Error> {
    let database_url = if let Some(ref database_url) = config.database_url {
        grid.row([
            "Database URL".color(KEY_COLOUR),
            database_url.color(VAL_COLOUR),
        ]);
        database_url
    } else {
        grid.row([
            "Database URL".color(KEY_COLOUR),
            "undefined".color(ERR_COLOUR),
        ]);
        return Ok(None);
    };

    log::trace!("Connecting to: {}", database_url);

    let conn = match config.get_dbms_connection() {
        Err(msg) => {
            grid.row([
                "Connection error".color(ERR_COLOUR),
                format!("{}", msg).color(ERR_COLOUR),
            ]);
            return Ok(None);
        }
        Ok(connection) => {
            grid.row(["Connection".color(KEY_COLOUR), "working".color(VAL_COLOUR)]);
            connection
        }
    };

    grid.row([
        "Catalog".color(KEY_COLOUR),
        conn.get_catalog().color(VAL_COLOUR),
    ]);

    Ok(Some(conn))
}

// pub fn migrations(grid: &mut Grid2, config: &Config, conn: Option<&Box<Connection>>) -> MainResult {
//     let migrations = migration::read_migrations(config);
//
//     let last_deployed = if let Some(ref conn) = conn {
//         conn.get_last_deployed_migration()?
//     } else {
//         None
//     };
//
//     grid.row(["", ""]);
//     grid.row(["Migrations:".color("cyan").bold(), "".color("white")]);
//     grid.row(["", ""]);
//
//     for m in migrations {
//         let is_deployed: Option<bool> = if conn.is_some() {
//             Some(if let Some(last_deployed) = last_deployed {
//                 if let Some(id) = m.get_id().get_id() {
//                     last_deployed >= id
//                 } else {
//                     false
//                 }
//             } else {
//                 false
//             })
//         } else {
//             None
//         };
//
//         let seal = format!(
//             "{}",
//             if m.is_sealed() {
//                 "  ".color("")
//             } else {
//                 "* ".color("green")
//             }
//         );
//
//         let name = format!("{}", m.get_id()).color(if !m.is_sealed() {
//             "white"
//         } else if let Some(deployed) = is_deployed {
//             if deployed {
//                 "blue"
//             } else {
//                 "green"
//             }
//         } else {
//             "white"
//         });
//
//         let status = format!(
//             "{}",
//             if !m.is_sealed() {
//                 "[ ]".color("white")
//             } else if let Some(deployed) = is_deployed {
//                 if deployed {
//                     "[+]".color("blue")
//                 } else {
//                     "[ ]".color("green")
//                 }
//             } else {
//                 "[?]".color("white")
//             }
//         );
//
//         grid.row([format!(" {}{}", seal, name), format!("{}", status)]);
//     }
//
//     Ok(())
// }
