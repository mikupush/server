.PHONY: build-linux build-linux-release

build-linux:
	cargo build \
	&& scripts/deb-package.sh debug \
	&& scripts/tar-package.sh debug

build-linux-release:
	cargo build --release \
	&& scripts/deb-package.sh release \
    && scripts/tar-package.sh release
