# Project variables
PROJECT_NAME = storage-engine
CARGO = cargo

# Directories
SRC_DIR = src
PROTO_DIR = proto
TARGET_DIR = target

# Clippy configuration
CLIPPY_OPTS = -- -D warnings

.PHONY: all build check test lint lint-fix clean run help doc

# Default target
all: lint build test

# Build the project
build:
	@echo "Building $(PROJECT_NAME)..."
	@$(CARGO) build

# Check if the project compiles
check:
	@echo "Checking $(PROJECT_NAME)..."
	@$(CARGO) check

# Run tests
test:
	@echo "Running tests..."
	@$(CARGO) test

# Run clippy lints
lint:
	@echo "Running clippy..."
	@$(CARGO) clippy $(CLIPPY_OPTS)
	@echo "Running rustfmt check..."
	@$(CARGO) fmt -- --check

# Fix linting issues automatically where possible
lint-fix:
	@echo "Fixing clippy warnings..."
	@$(CARGO) clippy --fix --allow-dirty $(CLIPPY_OPTS)
	@echo "Formatting code..."
	@$(CARGO) fmt

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@$(CARGO) clean
	@rm -rf $(TARGET_DIR)

# Run the server
run:
	@echo "Running server..."
	@$(CARGO) run

# Run the client example
run-client:
	@echo "Running client..."
	@$(CARGO) run --example grpc_client --features client

# Update dependencies
update:
	@echo "Updating dependencies..."
	@$(CARGO) update

# Show help
help:
	@echo "Available targets:"
	@echo "  all        - Run lint, build, and test"
	@echo "  build      - Build the project"
	@echo "  check      - Check if the project compiles"
	@echo "  test       - Run tests"
	@echo "  lint       - Run clippy lints and format checks"
	@echo "  lint-fix   - Fix linting issues automatically"
	@echo "  clean      - Clean build artifacts"
	@echo "  run        - Run the server"
	@echo "  run-client - Run the client example"
	@echo "  update     - Update dependencies"
	@echo "  help       - Show this help message"

# Generate documentation
doc:
	@echo "Generating documentation..."
	@$(CARGO) doc --no-deps --open