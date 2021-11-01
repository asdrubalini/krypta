test-all:
	cargo test -p vault-manager -p fs -p crypto --release -- --nocapture
