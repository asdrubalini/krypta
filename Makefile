test-all:
	cargo test -p vault-manager -p metadata-fs --release -- --nocapture
