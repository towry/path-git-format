all:
	echo "..."

release:
	cargo build --release

install: release
	cp ./target/release/path-git-format /usr/local/bin/

.PHONY: release install
