use std::sync::Arc;

use crate::{config::Configuration, database::Database};
use clap::{App, Arg};

use super::{status, sync};

#[derive(Debug)]
pub enum CliCommand {
    Sync { path: String },
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
                    .long("sync")
                    .takes_value(true)
                    .value_name("path"),
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
            let path = matches.value_of("sync").unwrap();
            return Some(CliCommand::Sync {
                path: path.to_owned(),
            });
        } else if matches.is_present("status") {
            return Some(CliCommand::Status);
        }

        None
    }

    pub async fn execute(self, config: Arc<Configuration>, database: &Database) {
        match self {
            CliCommand::Sync { path } => sync::execute(database, path).await,
            CliCommand::Status => status::execute(database).await,
        };
    }
}
