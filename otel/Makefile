.PHONY: all build test
.DEFAULT_GOAL: build

build:
	@echo "Building..."
	cargo build
test: build
	@echo "Running all tests..."
	php ~/run-tests.php -q --show-diff tests/
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