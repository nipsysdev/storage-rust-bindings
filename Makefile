# Makefile for Codex Rust Bindings

NIM_CODEX_DIR := vendor/nim-codex
NIM_CODEX_LIB_DIR   := $(abspath $(NIM_CODEX_DIR)/library)
NIM_CODEX_BUILD_DIR := $(abspath $(NIM_CODEX_DIR)/build)

.PHONY: all build test clean example docs install submodules libcodex

# Default target
all: build

# Initialize git submodules
submodules:
	@echo "Fetching submodules..."
	@git submodule update --init --recursive

# Update submodules
update: | submodules
	@echo "Updating nim-codex..."
	@$(MAKE) -C $(NIM_CODEX_DIR) update

# Build libcodex from source (static library to avoid linking issues)
libcodex: | submodules
	@echo "Building static libcodex..."
	@$(MAKE) -C $(NIM_CODEX_DIR) STATIC=1 libcodex

# Build the library
build: | libcodex
	cargo build --release

# Build with debug information
build-debug: | libcodex
	cargo build

# Run tests
test: | libcodex
	cargo test

# Run integration tests
test-integration: | libcodex
	cargo test --test integration_test

# Run example
example: | libcodex
	cargo run --example basic_usage

# Clean build artifacts
clean:
	cargo clean
	@git submodule deinit -f $(NIM_CODEX_DIR)

# Generate documentation
docs:
	cargo doc --no-deps --open

# Install the library
install: build
	cargo install --path .

# Check code formatting
fmt:
	cargo fmt --all

# Run linter
clippy:
	cargo clippy -- -D warnings

# Run all checks
check: fmt clippy test

# Build with all features
build-all: | libcodex
	cargo build --all-features

# Test with all features
test-all: | libcodex
	cargo test --all-features

# Prepare for release
release-prep: check build-all test-all
	@echo "Ready for release!"