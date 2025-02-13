.PHONY: all build test
.DEFAULT_GOAL: build

build:
	@echo "Building..."
	cargo build --release
test: build
	@echo "Running tests..."
	php ~/run-tests.php -q --show-diff tests/phpt/
export: build
	@echo "Running live export tests..."
	php ~/run-tests.php -q --show-diff tests/otlp/