.PHONY: all build test
.DEFAULT_GOAL: build

build:
	@echo "Building..."
	cargo build
test: build
	@echo "Running tests..."
	php ~/run-tests.php -q --show-diff tests/phpt/
export: build
	@echo "Running live export tests..."
	php ~/run-tests.php -q --show-diff tests/otlp/
auto: build
	@echo "Running auto-instrumentation tests..."
	php ~/run-tests.php -q --show-diff tests/auto/