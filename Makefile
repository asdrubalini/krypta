test-all:
	cargo test -p vault-manager -p metadata-fs -p crypto --release -- --nocapture
