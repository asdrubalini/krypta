test-all:
	cargo test -p vault-manager-impl -p fs -p crypto -p tmp --release -- --nocapture

