.PHONY: setup build test lint fmt run container-build container-test

setup:
	cargo fetch

build:
	cargo build

test:
	cargo test

lint:
	cargo clippy --all-targets --all-features -- -D warnings

fmt:
	cargo fmt --all

run:
	cargo run -- $(ARGS)

container-build:
	docker build -t devopster-cli-dev -f .devcontainer/Dockerfile .

container-test:
	docker build -t devopster-cli-ci .
	docker run --rm devopster-cli-ci make test
