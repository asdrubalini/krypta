export DATABASE_FILE=/tmp/files.db
cargo build --release
./target/release/krypta debug
./target/release/krypta sync
./target/release/krypta encrypt
