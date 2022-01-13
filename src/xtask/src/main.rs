use std::{env, fs::remove_file, process::Command};

fn main() {
    let mut args = env::args();
    args.next().unwrap();

    match args.next().unwrap().as_str() {
        "test-full" => test_full(),
        _ => panic!("xtask: invalid argument"),
    };
}

fn exec(command: impl AsRef<str>) {
    let command = command.as_ref();
    let mut command = command.split_ascii_whitespace();

    let bin = command.next().unwrap();
    let arg = command.collect::<Vec<_>>();

    Command::new(bin)
        .args(&arg)
        .spawn()
        .unwrap_or_else(|_| panic!("failed to execute {bin}"))
        .wait_with_output()
        .unwrap();
}

fn test_full() {
    let database = "/tmp/files.db";
    env::set_var("DATABASE_FILE", database);
    env::set_var("RUST_LOG", "trace");
    remove_file(database).ok();

    [
        "cargo build --release",
        "target/release/krypta debug",
        "target/release/krypta sync",
        "target/release/krypta encrypt",
    ]
    .into_iter()
    .map(exec)
    .count();
}
