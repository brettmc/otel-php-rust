.PHONY: all build test
.DEFAULT_GOAL: build
EXTENSION_NAME = otel
EXTENSION_FILE = target/release/lib${EXTENSION_NAME}.so
EXTENSION_INSTALL_PATH = $(shell php-config --extension-dir)/${EXTENSION_NAME}.so

build:
	@echo "Building..."
	cargo build --release
	cp target/release/libotel.so modules/otel.so
build-test:
	# phpize is only required to provide run-tests.php
	@echo "Building for test..."
	phpize
	cargo build --features test
	cp target/debug/libotel.so modules/otel.so
re: build-test
	php -d extension=target/debug/libotel.so --re otel
ri: build-test
	php -d extension=target/debug/libotel.so --ri otel
check:
	cargo check
install: build
	@echo "Installing..."
	cp ${EXTENSION_FILE} ${EXTENSION_INSTALL_PATH}
	@echo "✅ Installed $(EXTENSION_NAME) to $(EXTENSION_INSTALL_PATH)"
	@echo "📌 To enable it, add 'extension=$(EXTENSION_NAME).so' to php.ini"
clean:
	cargo clean
	rm -rf configure* config.h* libtool Makefile.* run-tests.php build/ autom4te.cache/
	rm -rf acinclude.m4 config.log config.guess config.nice config.status config.sub install-sh ltmain.sh missing mkinstalldirs
	rm -rf tests/auto/laminas/vendor/* tests/auto/laminas/composer.lock
	rm -rf tests/auto/zf1/vendor/* tests/auto/zf1/composer.lock
	rm -rf tests/auto/psr18/vendor/* tests/auto/psr18/composer.lock
update:
	@echo "Updating composer dependencies..."
	(cd tests/auto/laminas && composer update --no-dev)
	(cd tests/auto/zf1 && composer update --no-dev)
	(cd tests/auto/psr18 && composer update --no-dev)
	@echo "✅ Composer dependencies updated"
clean-build-test: clean test-all
test: build-test
	@echo "Running tests..."
	cargo test
	php run-tests.php -q tests/
test-phpt: build-test
	@echo "Running phpt tests..."
	php run-tests.php -q --show-diff tests/phpt/
test-export: build-test
	@echo "Running live export tests..."
	php run-tests.php -q --show-diff tests/otlp/
test-auto: build-test
	@echo "Running auto-instrumentation tests..."
	php run-tests.php -q --show-diff tests/auto/
test-http: build-test
	@echo "Running HTTP tests with built-in webserver..."
	php run-tests.php -q --show-diff tests/http/
