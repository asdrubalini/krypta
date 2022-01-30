use std::{env, mem::forget, process::Command};

use rand::{prelude::SmallRng, SeedableRng};
use tmp::{RandomFill, Tmp};

fn main() {
    let mut args = env::args();
    args.next().unwrap();

    match args.next().unwrap().as_str() {
        "populate-unlocked" => populate_unlocked(),
        "coverage" => coverage(),
        _ => panic!("xtask: invalid argument"),
    };
}

fn exec(command: impl AsRef<str>) {
    let command = command.as_ref();
    println!("Running {command}");
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

fn coverage() {
    exec("cargo tarpaulin --workspace --release --skip-clean");
}

fn populate_unlocked() {
    let tmp = Tmp::random();

    println!("Generating random files at {:?}", tmp.base_path());

    let mut rng = SmallRng::seed_from_u64(0);
    tmp.random_fill(25_000, &mut rng);

    println!("Done generating random files ");

    forget(tmp);
}
