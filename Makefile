# Path to the environment file
ENV_FILE ?= .env

# Cargo commands
CARGO := cargo
CARGO_TEST := $(CARGO) test
CARGO_FMT := $(CARGO) fmt
CARGO_CLIPPY := $(CARGO) clippy

.PHONY: fmt-check clippy run

test: unit integration

fmt-check:
	$(CARGO_FMT) -- --check

clippy:
	$(CARGO_CLIPPY) --all-targets --all-features -- -D warnings

run:
	@set -a; \
	source $(ENV_FILE); \
	set +a; \
	$(CARGO) run
