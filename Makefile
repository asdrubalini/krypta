test-all:
	cargo test -p vault-manager-impl -p fs -p crypto --release -- --nocapture
