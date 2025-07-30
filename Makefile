.PHONY: build bash
.DEFAULT_GOAL: bash

build-image:
	@echo "Building image..."
	docker compose build
build:
	@echo "Building extension..."
	docker compose run --rm php make build-test
bash:
	docker compose run --rm php bash
clean:
	@echo "Cleaning up..."
	docker compose run --rm php make clean