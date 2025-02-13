.PHONY: build bash
.DEFAULT_GOAL: bash

build:
	@echo "Building..."
	docker compose build
bash:
	docker compose run --rm php bash