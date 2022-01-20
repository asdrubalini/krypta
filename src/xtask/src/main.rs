use std::{env, fs::remove_file, mem::forget, process::Command};

use rand::{thread_rng, Rng};
use tmp::{RandomFill, Tmp};

fn main() {
    let mut args = env::args();
    args.next().unwrap();

    match args.next().unwrap().as_str() {
        "test-full" => test_full(),
        "populate-unlocked" => populate_unlocked(),
        _ => panic!("xtask: invalid argument"),
    };
}

fn exec(command: impl AsRef<str>) {
    let command = command.as_ref();
    let mut command = command.split_ascii_whitespace();

    let bin = command.next().unwrap();
    let arg = command.collect::<Vec<_>>();

    let status = Command::new(bin)
        .args(&arg)
        .spawn()
        .unwrap_or_else(|_| panic!("failed to execute {bin}"))
        .wait()
        .unwrap();

    assert!(status.success());
}

fn test_full() {
    let database = "/tmp/files.db";
    env::set_var("DATABASE_FILE", database);
    env::set_var("RUST_LOG", "trace");
    remove_file(database).ok();

    [
        "cargo build --release",
        "cargo run --release -q -- debug",
        "cargo run --release -q -- sync",
        "cargo run --release -q -- encrypt",
    ]
    .into_iter()
    .map(exec)
    .count();
}

fn populate_unlocked() {
    let tmp = Tmp::with_path("/krypta/unlocked/");

    tmp.random_fill(50_000, || {
        if thread_rng().gen_bool(0.6) {
            thread_rng().gen_range(10..8192)
        } else {
            thread_rng().gen_range(50_000..100_000)
        }
    });

    forget(tmp);
}
