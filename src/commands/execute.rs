use std::sync::Arc;

use crate::{config::Configuration, database::Database};
use clap::{App, Arg};

use super::{status, sync};

#[derive(Debug)]
pub enum CliCommand {
    Sync,
    Status,
}

impl CliCommand {
    pub fn try_parse() -> Option<CliCommand> {
        let matches = App::new("vault-manager")
            .version("0.1.0")
            .author("Asdrubalini <asdrubalini@mail.com>")
            .about("Hide files and metadata from cloud storages")
            .arg(
                Arg::with_name("sync")
                    .help("sync a folder into the database")
                    .long("sync"),
            )
            .arg(
                Arg::with_name("status")
                    .help("get database status")
                    .short("s")
                    .long("status")
                    .takes_value(false),
            )
            .get_matches();

        if matches.is_present("sync") {
            return Some(CliCommand::Sync);
        } else if matches.is_present("status") {
            return Some(CliCommand::Status);
        }

        None
    }

    pub async fn execute(self, config: Arc<Configuration>, database: &Database) {
        let config = config.as_ref();

        match self {
            CliCommand::Sync => sync::execute(database, config).await,
            CliCommand::Status => status::execute(database).await,
        };
    }
}
