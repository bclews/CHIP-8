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
	@echo ""
	@echo "Testing:"
	@echo "  test          Run all tests"
	@echo "  test-unit     Run unit tests only"
	@echo "  test-lib      Run library tests only"
	@echo "  test-verbose  Run tests with verbose output"
	@echo "  coverage      Generate test coverage report (requires cargo-tarpaulin)"
	@echo ""
	@echo "Code Quality:"
	@echo "  check         Check code without building"
	@echo "  clippy        Run clippy linter"
	@echo "  fmt           Format code with rustfmt"
	@echo "  fmt-check     Check if code is formatted correctly"
	@echo "  audit         Check dependencies for security vulnerabilities"
	@echo ""
	@echo "Documentation:"
	@echo "  doc           Generate documentation"
	@echo "  doc-open      Generate and open documentation"
	@echo ""
	@echo "Benchmarking:"
	@echo "  bench         Run benchmarks"
	@echo "  bench-cpu     Run CPU benchmarks only"
	@echo ""
	@echo "Development:"
	@echo "  run           Run the emulator (requires ROM file argument)"
	@echo "  run-example   Run with the test ROM"
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

# Testing targets
.PHONY: test
test:
	@echo "🧪 Running all tests..."
	$(CARGO) test

.PHONY: test-unit
test-unit:
	@echo "🧪 Running unit tests..."
	$(CARGO) test --lib

.PHONY: test-lib
test-lib: test-unit

.PHONY: test-verbose
test-verbose:
	@echo "🧪 Running tests with verbose output..."
	$(CARGO) test -- --nocapture

.PHONY: test-ignored
test-ignored:
	@echo "🧪 Running ignored tests..."
	$(CARGO) test -- --ignored

.PHONY: coverage
coverage:
	@echo "📊 Generating test coverage report..."
	@if command -v cargo-tarpaulin >/dev/null 2>&1; then \
		$(CARGO) tarpaulin --out Html --output-dir coverage; \
		echo "Coverage report generated in coverage/tarpaulin-report.html"; \
	else \
		echo "❌ cargo-tarpaulin not found. Install with: cargo install cargo-tarpaulin"; \
		exit 1; \
	fi

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

.PHONY: audit
audit:
	@echo "🔒 Auditing dependencies..."
	@if command -v cargo-audit >/dev/null 2>&1; then \
		$(CARGO) audit; \
	else \
		echo "❌ cargo-audit not found. Install with: cargo install cargo-audit"; \
		exit 1; \
	fi

# Documentation targets
.PHONY: doc
doc:
	@echo "📚 Generating documentation..."
	$(CARGO) doc --no-deps

.PHONY: doc-open
doc-open:
	@echo "📚 Generating and opening documentation..."
	$(CARGO) doc --no-deps --open

# Benchmarking targets
.PHONY: bench
bench:
	@echo "⚡ Running benchmarks..."
	$(CARGO) bench

.PHONY: bench-cpu
bench-cpu:
	@echo "⚡ Running CPU benchmarks..."
	$(CARGO) bench --bench cpu_benchmark

# Development targets
.PHONY: run
run: build
	@if [ -z "$(ROM)" ]; then \
		echo "❌ ROM file required. Usage: make run ROM=path/to/rom.ch8"; \
		exit 1; \
	fi
	@echo "🎮 Running Chip-8 emulator with $(ROM)..."
	$(CARGO) run -- "$(ROM)"

.PHONY: run-example
run-example: build
	@echo "🎮 Running Chip-8 emulator with test ROM..."
	$(CARGO) run -- test_rom.ch8

.PHONY: run-release
run-release: release
	@if [ -z "$(ROM)" ]; then \
		echo "❌ ROM file required. Usage: make run-release ROM=path/to/rom.ch8"; \
		exit 1; \
	fi
	@echo "🎮 Running Chip-8 emulator (release) with $(ROM)..."
	./$(RELEASE_DIR)/$(BINARY_NAME) "$(ROM)"

.PHONY: watch
watch:
	@echo "👀 Watching for changes..."
	@if command -v cargo-watch >/dev/null 2>&1; then \
		$(CARGO) watch -x build; \
	else \
		echo "❌ cargo-watch not found. Install with: cargo install cargo-watch"; \
		exit 1; \
	fi

.PHONY: watch-test
watch-test:
	@echo "👀 Watching for changes and running tests..."
	@if command -v cargo-watch >/dev/null 2>&1; then \
		$(CARGO) watch -x test; \
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
	rm -rf coverage/
	rm -f test_rom.ch8
	rm -f *.ch8

.PHONY: update
update:
	@echo "📦 Updating dependencies..."
	$(CARGO) update

.PHONY: outdated
outdated:
	@echo "📦 Checking for outdated dependencies..."
	@if command -v cargo-outdated >/dev/null 2>&1; then \
		$(CARGO) outdated; \
	else \
		echo "❌ cargo-outdated not found. Install with: cargo install cargo-outdated"; \
		exit 1; \
	fi

# CI/CD targets
.PHONY: ci-check
ci-check: fmt-check clippy test
	@echo "✅ All CI checks passed!"

.PHONY: ci-full
ci-full: clean fmt-check clippy test doc bench
	@echo "✅ Full CI pipeline completed!"

# Development setup targets
.PHONY: dev-setup
dev-setup:
	@echo "🛠️  Setting up development environment..."
	@echo "Installing cargo tools..."
	$(CARGO) install cargo-watch cargo-audit cargo-tarpaulin cargo-outdated
	@echo "✅ Development environment setup complete!"

# ROM management targets
.PHONY: create-test-rom
create-test-rom:
	@echo "🎮 Creating test ROM..."
	@echo -e '\x61\x23\x62\x45\x81\x24\xA3\x00\x71\x05' > test_rom.ch8
	@echo "✅ Test ROM created: test_rom.ch8"

# Size analysis
.PHONY: size
size: release
	@echo "📏 Binary size analysis:"
	@ls -lh $(RELEASE_DIR)/$(BINARY_NAME) | awk '{print "Release binary: " $$5}'
	@if [ -f $(DEBUG_DIR)/$(BINARY_NAME) ]; then \
		ls -lh $(DEBUG_DIR)/$(BINARY_NAME) | awk '{print "Debug binary:   " $$5}'; \
	fi

# Performance profiling (requires external tools)
.PHONY: profile
profile: release
	@echo "🔬 Profiling application..."
	@if command -v perf >/dev/null 2>&1; then \
		echo "Running perf profiling..."; \
		perf record --call-graph=dwarf ./$(RELEASE_DIR)/$(BINARY_NAME) test_rom.ch8; \
		perf report; \
	elif command -v instruments >/dev/null 2>&1; then \
		echo "Running Instruments profiling (macOS)..."; \
		instruments -t "Time Profiler" ./$(RELEASE_DIR)/$(BINARY_NAME) test_rom.ch8; \
	else \
		echo "❌ No profiling tools found (perf on Linux, instruments on macOS)"; \
		exit 1; \
	fi

# Linting and quality gates
.PHONY: lint
lint: fmt-check clippy
	@echo "✅ All linting checks passed!"

.PHONY: pre-commit
pre-commit: lint test
	@echo "✅ Pre-commit checks passed!"

# Quick development cycle
.PHONY: quick
quick: check test

.PHONY: full
full: clean lint test doc

# Ensure test ROM exists for run-example
test_rom.ch8:
	@$(MAKE) create-test-rom

# Make run-example depend on test ROM existing
run-example: test_rom.ch8

# Phony target to list all available targets
.PHONY: list
list:
	@$(MAKE) -pRrq -f $(lastword $(MAKEFILE_LIST)) : 2>/dev/null | awk -v RS= -F: '/^# File/,/^# Finished Make data base/ {if ($$1 !~ "^[#.]") {print $$1}}' | sort | egrep -v -e '^[^[:alnum:]]' -e '^$@$$'