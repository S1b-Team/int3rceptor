.PHONY: dev backend ui build test lint fmt docker

dev:
	cargo run -p cli -- --listen 127.0.0.1:8080 --api 127.0.0.1:3000

backend:
	cargo run -p api

ui:
	cd ui && npm install && npm run dev

build:
	cargo build --release

test:
	cargo test --all

lint:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt --all

docker:
	docker compose up --build
