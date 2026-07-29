.PHONY: build build-release test lint fmt clippy clean install

# Default target
all: build

# Build debug
build:
	cargo build

# Build release
build-release:
	cargo build --release

# Run tests
test:
	cargo test

# Run linter
lint: clippy fmt-check

# Format code
fmt:
	cargo fmt

# Check formatting
fmt-check:
	cargo fmt -- --check

# Run clippy
clippy:
	cargo clippy -- -D warnings

# Clean build artifacts
clean:
	cargo clean

# Install locally
install: build-release
	install -m 755 target/release/qcker /usr/local/bin/qcker

# Run integration tests (requires root or user namespaces)
test-integration: build
	@echo "=== Integration Tests ==="
	@echo "Preparing rootfs..."
	@mkdir -p /tmp/qcker-test/rootfs
	@if [ ! -f /tmp/qcker-test/rootfs/bin/sh ]; then \
		echo "Exporting alpine rootfs..."; \
		docker export $$(docker create alpine:latest) | tar -C /tmp/qcker-test/rootfs -xf -; \
	fi
	@echo "Running container lifecycle test..."
	cargo run -- create --rootfs /tmp/qcker-test/rootfs -- sleep 5
	@echo "Integration tests passed!"

# Help
help:
	@echo "Qcker - Container Engine"
	@echo ""
	@echo "Targets:"
	@echo "  build           Build debug binary"
	@echo "  build-release   Build release binary"
	@echo "  test            Run unit tests"
	@echo "  lint            Run linter (clippy + fmt check)"
	@echo "  fmt             Format code"
	@echo "  clippy          Run clippy linter"
	@echo "  clean           Clean build artifacts"
	@echo "  install         Install to /usr/local/bin"
	@echo "  test-integration  Run integration tests"
	@echo "  help            Show this help"
