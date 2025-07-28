.PHONY: build bash
.DEFAULT_GOAL: bash

build-image:
	@echo "Building image..."
	docker compose build
build:
	@echo "Building extension..."
	docker compose run --rm php make build
bash:
	docker compose run --rm php bash