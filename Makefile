test-all:
	cargo test -p vault-manager-impl -p fs -p crypto -p temp-path --release -- --nocapture

