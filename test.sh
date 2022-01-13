export DATABASE_FILE=/tmp/files.db

rm $DATABASE_FILE
cargo build --release
./target/release/krypta debug
./target/release/krypta sync
./target/release/krypta encrypt
