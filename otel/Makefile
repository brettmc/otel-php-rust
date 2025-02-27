.PHONY: all build test
.DEFAULT_GOAL: build
EXTENSION_NAME = otel
EXTENSION_FILE = target/release/lib${EXTENSION_NAME}.so
EXTENSION_INSTALL_PATH = $(shell php-config --extension-dir)/${EXTENSION_NAME}.so

build:
	@echo "Building..."
	cargo build --release
	cp target/release/libotel.so modules/otel.so
install: build
	@echo "Installing..."
	cp ${EXTENSION_FILE} ${EXTENSION_INSTALL_PATH}
	@echo "✅ Installed $(EXTENSION_NAME) to $(EXTENSION_INSTALL_PATH)"
	@echo "📌 To enable it, add 'extension=$(EXTENSION_NAME).so' to php.ini"
clean:
	cargo clean
test: build
	@echo "Running all tests..."
	php ~/run-tests.php -q tests/
test-func: build
	@echo "Running functional tests..."
	php ~/run-tests.php -q --show-diff tests/phpt/
test-export: build
	@echo "Running live export tests..."
	php ~/run-tests.php -q --show-diff tests/otlp/
test-auto: build
	@echo "Running auto-instrumentation tests..."
	php ~/run-tests.php -q --show-diff tests/auto/
test-http: build
	@echo "Running HTTP tests with built-in webserver..."
	php ~/run-tests.php -q --show-diff tests/http/
