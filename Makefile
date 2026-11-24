.PHONY: dev

# Run the project locally: make dev  (or: make dev ARGS="--json experience")
dev:
	cargo run -- $(ARGS)
