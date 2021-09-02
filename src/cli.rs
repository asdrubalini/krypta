use clap::{App, Arg};

#[derive(Debug)]
pub enum CliCommand {
    Sync { path: String },
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
                    .short("s")
                    .long("sync")
                    .takes_value(true)
                    .value_name("path"),
            )
            .get_matches();

        if matches.is_present("sync") {
            let path = matches.value_of("sync").unwrap();
            return Some(CliCommand::Sync {
                path: path.to_owned(),
            });
        }

        None
    }
}
