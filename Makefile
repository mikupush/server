.PHONY: web \
	server-debug \
	server-release \
	linux-debug \
	linux-release

web:
	npm run build

server-debug:
	cargo build

server-release:
	cargo build

linux-debug: web server-debug
	scripts/deb-package.sh debug && scripts/tar-package.sh debug

linux-release: web server-release
	scripts/deb-package.sh release && scripts/tar-package.sh release
