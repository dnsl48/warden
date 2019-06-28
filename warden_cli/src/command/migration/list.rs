use crate::args::Args;
use crate::grid::{Grid, Grid2};
use crate::migration;
use crate::MainResult;
use colored::*;
use warden_core::config::Config;
use warden_core::dbms::Connection;

pub fn run(args: &Args) -> MainResult {
    let config = args.get_config()?;

    let mut grid: Grid2 = Grid::default();

    migrations(&mut grid, &config, None)?;

    grid.row(["", ""]);

    print!("{}", grid.display());

    Ok(())
}

pub fn migrations(grid: &mut Grid2, config: &Config, conn: Option<&Box<Connection>>) -> MainResult {
    let migrations = migration::read_migrations(config);

    let last_deployed = if let Some(ref conn) = conn {
        conn.get_last_deployed_migration()?
    } else {
        None
    };

    grid.row(["", ""]);
    grid.row(["Migrations:".color("cyan").bold(), "".color("white")]);
    grid.row(["", ""]);

    for m in migrations {
        let is_deployed: Option<bool> = if conn.is_some() {
            Some(if let Some(last_deployed) = last_deployed {
                if let Some(id) = m.get_id().get_id() {
                    last_deployed >= id
                } else {
                    false
                }
            } else {
                false
            })
        } else {
            None
        };

        let seal = format!(
            "{}",
            if m.is_sealed() {
                "  ".color("")
            } else {
                "* ".color("green")
            }
        );

        let name = format!("{}", m.get_id()).color(if !m.is_sealed() {
            if conn.is_some() {
                "red"
            } else {
                "green"
            }
        } else if let Some(deployed) = is_deployed {
            if deployed {
                "blue"
            } else {
                "green"
            }
        } else {
            if conn.is_some() {
                "white"
            } else {
                "blue"
            }
        });

        let status = format!(
            "{}",
            if !m.is_sealed() {
                if conn.is_some() {
                    "[ ]".color("white")
                } else {
                    "".color("white")
                }
            } else if let Some(deployed) = is_deployed {
                if deployed {
                    "[+]".color("blue")
                } else {
                    "[ ]".color("green")
                }
            } else {
                "".color("white")
            }
        );

        grid.row([format!(" {}{}", seal, name), format!("{}", status)]);
    }

    Ok(())
}
