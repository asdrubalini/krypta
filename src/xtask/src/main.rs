use std::{env, fs::remove_file, process::Command, thread};

use rand::{thread_rng, Rng};
use tmp::{RandomFill, Tmp};

fn main() {
    let mut args = env::args();
    args.next().unwrap();

    match args.next().unwrap().as_str() {
        "integration-tests" => integration_tests(),
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

fn integration_tests() {
    let database = "/tmp/krypta-integration-tests.db";
    env::set_var("DATABASE_FILE", database);
    env::set_var("RUST_LOG", "trace");
    remove_file(database).ok();

    // Build and generate files in parallel
    let unlocked_tmp_handle = thread::spawn(populate_unlocked);
    exec("cargo build --release");

    let unlocked_tmp = unlocked_tmp_handle.join().unwrap();
    let locked_tmp = Tmp::empty();

    exec(format!(
        "./target/release/krypta set-unlocked {}",
        unlocked_tmp.path().to_string_lossy()
    ));
    exec(format!(
        "./target/release/krypta set-locked {}",
        locked_tmp.path().to_string_lossy()
    ));

    [
        "./target/release/krypta sync",
        "./target/release/krypta sync",
        "./target/release/krypta encrypt",
        "./target/release/krypta encrypt",
    ]
    .into_iter()
    .map(exec)
    .count();
}

fn populate_unlocked() -> Tmp {
    let tmp = Tmp::empty();

    println!("Generating random files at {:?}", tmp.path());

    tmp.random_fill(25_000, || {
        if thread_rng().gen_bool(0.9) {
            thread_rng().gen_range(10..8192)
        } else {
            thread_rng().gen_range(50_000..100_000)
        }
    });

    println!("Done generating random files ");

    tmp
}
