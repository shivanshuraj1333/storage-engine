# Project variables
PROJECT_NAME = storage-engine
CARGO = cargo

# Directories
SRC_DIR = src
PROTO_DIR = proto
TARGET_DIR = target
SCRIPTS_DIR = scripts

# Clippy configuration
CLIPPY_OPTS = -- -D warnings

.PHONY: all build check test lint lint-fix clean run help doc setup-proto

# Default target
all: lint build test

# Setup proto files
setup-proto:
	@echo "Setting up OpenTelemetry proto files..."
	@chmod +x $(SCRIPTS_DIR)/fetch-protos.sh
	@$(SCRIPTS_DIR)/fetch-protos.sh
	@echo "Proto files downloaded successfully"

# Build the project
build: setup-proto
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
	@echo "Cleaning proto files..."
	@rm -rf $(PROTO_DIR)/opentelemetry

# Run the server
run:
	@echo "Running server..."
	@$(CARGO) run

# Run the client example with logging
run-client:
	@echo "Running client..."
	@RUST_LOG=info cargo run --example grpc_client --features client

# Generate documentation
doc:
	@echo "Generating documentation..."
	@$(CARGO) doc --no-deps

# Help command
help:
	@echo "Available commands:"
	@echo "  make setup-proto  - Download and setup OpenTelemetry proto files"
	@echo "  make build       - Build the project"
	@echo "  make test        - Run tests"
	@echo "  make lint        - Run clippy and format checks"
	@echo "  make lint-fix    - Fix linting issues"
	@echo "  make clean       - Clean build artifacts"
	@echo "  make run         - Run the server"
	@echo "  make run-client  - Run the test client"
	@echo "  make doc         - Generate documentation"