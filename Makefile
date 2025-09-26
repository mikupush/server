.PHONY: build build-release

build:
	cargo build \
	&& scripts/deb-package.sh debug

build-release:
	cargo build --release \
	&& scripts/deb-package.sh release
