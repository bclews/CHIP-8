# Chip-8 Emulator Makefile
# A modern Rust Chip-8 emulator with comprehensive build automation

# Variables
CARGO := cargo
BINARY_NAME := chip8
TARGET_DIR := target
RELEASE_DIR := $(TARGET_DIR)/release
DEBUG_DIR := $(TARGET_DIR)/debug
DOCS_DIR := $(TARGET_DIR)/doc

# Default target
.PHONY: all
all: build

# Help target - shows available commands
.PHONY: help
help:
	@echo "Chip-8 Emulator - Available Commands:"
	@echo ""
	@echo "Building:"
	@echo "  build         Build the project in debug mode"
	@echo "  release       Build the project in release mode"
	@echo "  install       Install the binary to ~/.cargo/bin"
	@echo "  uninstall     Uninstall the binary from ~/.cargo/bin"
	@echo ""
	@echo "Testing:"
	@echo "  test          Run all tests"
	@echo "  test-unit     Run unit tests only"
	@echo "  test-verbose  Run tests with verbose output"
	@echo ""
	@echo "Code Quality:"
	@echo "  check         Check code without building"
	@echo "  clippy        Run clippy linter"
	@echo "  fmt           Format code with rustfmt"
	@echo "  fmt-check     Check if code is formatted correctly"
	@echo "  lint          Run all linting checks (fmt-check + clippy)"
	@echo "  pre-commit    Run pre-commit checks (lint + test)"
	@echo ""
	@echo "Documentation:"
	@echo "  doc-open      Generate and open documentation"
	@echo ""
	@echo "Benchmarking:"
	@echo "  bench         Run benchmarks"
	@echo ""
	@echo "Development:"
	@echo "  run           Run the emulator (requires ROM file argument)"
	@echo "  watch         Watch for changes and rebuild"
	@echo "  deps          Show dependency tree"
	@echo ""
	@echo "Maintenance:"
	@echo "  clean         Clean build artifacts"
	@echo "  clean-all     Clean all generated files including docs"
	@echo "  update        Update dependencies"
	@echo ""
	@echo "Examples:"
	@echo "  make run ROM=game.ch8"
	@echo "  make test-verbose"
	@echo "  make release"

# Building targets
.PHONY: build
build:
	@echo "🔨 Building Chip-8 emulator (debug)..."
	$(CARGO) build

.PHONY: release
release:
	@echo "🚀 Building Chip-8 emulator (release)..."
	$(CARGO) build --release

.PHONY: install
install: release
	@echo "📦 Installing Chip-8 emulator..."
	$(CARGO) install --path .

.PHONY: uninstall
uninstall:
	@echo "🗑️  Uninstalling Chip-8 emulator..."
	$(CARGO) uninstall $(BINARY_NAME)

# Testing targets
.PHONY: test
test:
	@echo "🧪 Running all tests..."
	$(CARGO) test

.PHONY: test-unit
test-unit:
	@echo "🧪 Running unit tests..."
	$(CARGO) test --lib

.PHONY: test-verbose
test-verbose:
	@echo "🧪 Running tests with verbose output..."
	$(CARGO) test -- --nocapture

# Code quality targets
.PHONY: check
check:
	@echo "🔍 Checking code..."
	$(CARGO) check

.PHONY: clippy
clippy:
	@echo "📎 Running clippy linter..."
	$(CARGO) clippy --all-targets --all-features -- -D warnings

.PHONY: fmt
fmt:
	@echo "🎨 Formatting code..."
	$(CARGO) fmt

.PHONY: fmt-check
fmt-check:
	@echo "🎨 Checking code formatting..."
	$(CARGO) fmt -- --check

# Documentation targets
.PHONY: doc-open
doc-open:
	@echo "📚 Generating and opening documentation..."
	$(CARGO) doc --no-deps --open

# Benchmarking targets
.PHONY: bench
bench:
	@echo "⚡ Running benchmarks..."
	$(CARGO) bench

# Development targets
.PHONY: run
run: build
	@if [ -z "$(ROM)" ]; then \
		echo "❌ ROM file required. Usage: make run ROM=path/to/rom.ch8"; \
		exit 1; \
	fi
	@echo "🎮 Running Chip-8 emulator with $(ROM)..."
	$(CARGO) run -- "$(ROM)"

.PHONY: watch
watch:
	@echo "👀 Watching for changes..."
	@if command -v cargo-watch >/dev/null 2>&1; then \
		$(CARGO) watch -x build; \
	else \
		echo "❌ cargo-watch not found. Install with: cargo install cargo-watch"; \
		exit 1; \
	fi

.PHONY: deps
deps:
	@echo "🌳 Showing dependency tree..."
	$(CARGO) tree

# Maintenance targets
.PHONY: clean
clean:
	@echo "🧹 Cleaning build artifacts..."
	$(CARGO) clean

.PHONY: clean-all
clean-all: clean
	@echo "🧹 Cleaning all generated files..."
	rm -rf $(DOCS_DIR)
	rm -f test_rom.ch8
	rm -f *.ch8

.PHONY: update
update:
	@echo "📦 Updating dependencies..."
	$(CARGO) update

# Linting and quality gates
.PHONY: lint
lint: fmt-check clippy
	@echo "✅ All linting checks passed!"

.PHONY: pre-commit
pre-commit: lint test
	@echo "✅ Pre-commit checks passed!"

