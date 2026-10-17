.PHONY: all build build-test re ri check clippy install clean update clean-build-test \
	test test-all test-phpt test-export test-auto test-http
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
clippy:
	cargo clippy --all-targets --features test
install: build
	@echo "Installing..."
	cp ${EXTENSION_FILE} ${EXTENSION_INSTALL_PATH}
	@echo "✅ Installed $(EXTENSION_NAME) to $(EXTENSION_INSTALL_PATH)"
	@echo "📌 To enable it, add 'extension=$(EXTENSION_NAME).so' to php.ini"
clean:
	# in the dev containers target/ is a per-PHP-version volume, so it is a mount
	# point that cargo clean cannot remove; empty it in place instead
	@if mountpoint -q target 2>/dev/null; then \
		echo "target/ is a mount point, removing its contents"; \
		find target -mindepth 1 -delete; \
	else \
		cargo clean; \
	fi
	rm -rf configure* config.h* libtool Makefile.* run-tests.php build/ autom4te.cache/
	rm -rf acinclude.m4 config.log config.guess config.nice config.status config.sub install-sh ltmain.sh missing mkinstalldirs
	rm -rf tests/auto/laminas/vendor/* tests/auto/laminas/composer.lock
	rm -rf tests/auto/zf1/vendor/* tests/auto/zf1/composer.lock
	rm -rf tests/auto/psr18/vendor/* tests/auto/psr18/composer.lock
# Which fixtures can resolve depends on the running PHP, so this mirrors the gates in
# .github/workflows/test.yaml rather than duplicating the version matrix: zf1 always,
# laminas on 7.1+, psr18 on 7.2+. Running all three unconditionally fails on 7.0/7.1,
# where the skipped dirs have no resolvable release.
#
# 8.5 needs --ignore-platform-req=php for laminas: no laminas-mvc release supports it yet
# (3.8.0 caps at ~8.4.0), and without the flag the resolver falls back to mvc 2.x, which
# hard-requires laminas-form -- every matching release of which carries an unfixable XSS
# advisory that composer >=2.9 refuses to install. The flag keeps 3.8.0, which is what 8.4
# already tests against. It is scoped to 8.5 on purpose: on 7.x/8.0 it would force 3.8.0
# onto a PHP that cannot run it.
PHP_VERSION_ID := $(shell php -r 'echo PHP_VERSION_ID;')
update:
	@echo "Updating composer dependencies (PHP_VERSION_ID=$(PHP_VERSION_ID))..."
	(cd tests/auto/zf1 && composer update --no-dev)
ifeq ($(shell test $(PHP_VERSION_ID) -ge 80500 && echo yes),yes)
	(cd tests/auto/laminas && composer update --no-dev --ignore-platform-req=php)
else ifeq ($(shell test $(PHP_VERSION_ID) -ge 70100 && echo yes),yes)
	(cd tests/auto/laminas && composer update --no-dev)
else
	@echo "⏭  skipping laminas: needs PHP 7.1+"
endif
ifeq ($(shell test $(PHP_VERSION_ID) -ge 70200 && echo yes),yes)
	(cd tests/auto/psr18 && composer update --no-dev)
else
	@echo "⏭  skipping psr18: needs PHP 7.2+"
endif
	@echo "✅ Composer dependencies updated"
clean-build-test: clean test-all
test: build-test
	@echo "Running tests..."
	cargo test --features test
	php run-tests.php -q tests/
# the auto-instrumentation tests need their composer fixtures, which `clean`
# removes, so install them before running every suite
test-all: update test
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
