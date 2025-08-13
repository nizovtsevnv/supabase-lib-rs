# Justfile for Supabase Rust Client Library

# Default recipe
default: check

# Show all available commands
help:
    @just --list

# Run all pre-commit checks
check: format lint test build audit

# Setup pre-commit hooks
setup-precommit:
    @echo "🔧 Setting up pre-commit hooks..."
    ./scripts/setup-precommit.sh

# Run pre-commit hooks manually
precommit-run:
    @echo "🔧 Running pre-commit hooks..."
    pre-commit run --all-files

# Run specific pre-commit hook
precommit-run-hook HOOK:
    @echo "🔧 Running pre-commit hook: {{HOOK}}"
    pre-commit run {{HOOK}}

# Update pre-commit hooks to latest versions
precommit-update:
    @echo "📦 Updating pre-commit hooks..."
    pre-commit autoupdate

# Install pre-commit hooks
precommit-install:
    @echo "📦 Installing pre-commit hooks..."
    pre-commit install
    pre-commit install --hook-type commit-msg

# Format code
format:
    @echo "🔧 Formatting code..."
    cargo fmt

# Format code and check if changes were made
format-check:
    @echo "🔧 Checking code formatting..."
    cargo fmt --check

# Run clippy linter
lint:
    @echo "🔍 Running clippy linter..."
    cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
    @echo "🧪 Running tests..."
    cargo test

# Run tests with output
test-verbose:
    @echo "🧪 Running tests (verbose)..."
    cargo test -- --nocapture

# Run integration tests only
test-integration:
    @echo "🧪 Running integration tests..."
    cargo test --test '*' -- --nocapture

# Run unit tests only
test-unit:
    @echo "🧪 Running unit tests..."
    cargo test --lib --bins

# Run tests with coverage
coverage:
    @echo "📊 Running tests with coverage..."
    cargo tarpaulin --out Html --output-dir coverage

# Build the project
build:
    @echo "🔨 Building project..."
    cargo build

# Build in release mode
build-release:
    @echo "🔨 Building project (release)..."
    cargo build --release

# Security audit
audit:
    @echo "🔒 Running security audit..."
    cargo audit

# Check dependencies for issues
deny:
    @echo "🚫 Checking dependencies..."
    cargo deny check

# Update dependencies
update:
    @echo "⬆️ Updating dependencies..."
    cargo update

# Check for outdated dependencies
outdated:
    @echo "📅 Checking for outdated dependencies..."
    cargo outdated

# Clean build artifacts
clean:
    @echo "🧹 Cleaning build artifacts..."
    cargo clean

# Watch for file changes and run tests
watch:
    @echo "👀 Watching for changes..."
    cargo watch -x test

# Watch for file changes and run specific command
watch-cmd cmd:
    @echo "👀 Watching for changes and running: {{cmd}}"
    cargo watch -x "{{cmd}}"

# Generate documentation
docs:
    @echo "📚 Generating documentation..."
    cargo doc --no-deps --open

# Generate documentation without opening browser
docs-build:
    @echo "📚 Generating documentation..."
    cargo doc --no-deps

# Run benchmarks
bench:
    @echo "📈 Running benchmarks..."
    cargo bench

# Analyze binary size
bloat:
    @echo "📏 Analyzing binary size..."
    cargo bloat --release

# Install development tools
install-tools:
    @echo "🔧 Installing development tools..."
    cargo install cargo-audit cargo-deny cargo-outdated cargo-edit cargo-watch cargo-tarpaulin cargo-bloat

# Run example
example name:
    @echo "🚀 Running example: {{name}}"
    cargo run --example {{name}}

# Run all examples
examples:
    @echo "🚀 Running all examples..."
    @for example in basic_usage auth_example database_example storage_example realtime_example; do \
        echo "Running example: $example"; \
        cargo run --example $example || true; \
    done

# Setup development environment
setup:
    @echo "🛠️ Setting up development environment..."
    @if [ ! -f Cargo.lock ]; then echo "Generating Cargo.lock..."; cargo generate-lockfile; fi
    @echo "✅ Development environment ready!"

# Pre-commit hook (run before committing)
pre-commit: format-check lint test build
    @echo "✅ All pre-commit checks passed!"

# Release preparation
release-prep: clean format lint test build-release audit docs-build
    @echo "📦 Release preparation complete!"

# Start local Supabase for testing (requires Docker)
supabase-start:
    @echo "🚀 Starting local Supabase..."
    @if command -v supabase >/dev/null 2>&1; then \
        supabase start; \
    else \
        echo "❌ Supabase CLI not found. Install from: https://supabase.com/docs/guides/cli"; \
    fi

# Stop local Supabase
supabase-stop:
    @echo "🛑 Stopping local Supabase..."
    @if command -v supabase >/dev/null 2>&1; then \
        supabase stop; \
    else \
        echo "❌ Supabase CLI not found."; \
    fi

# Reset local Supabase
supabase-reset:
    @echo "🔄 Resetting local Supabase..."
    @if command -v supabase >/dev/null 2>&1; then \
        supabase db reset; \
    else \
        echo "❌ Supabase CLI not found."; \
    fi

# Nix-specific commands

# Build with Nix
nix-build:
    @echo "❄️ Building with Nix..."
    nix build

# Run checks with Nix
nix-check:
    @echo "❄️ Running Nix flake checks..."
    nix flake check

# Update Nix flake inputs
nix-update:
    @echo "❄️ Updating Nix flake inputs..."
    nix flake update

# Enter development shell
nix-shell:
    @echo "❄️ Entering Nix development shell..."
    nix develop

# Show flake info
nix-info:
    @echo "❄️ Showing flake information..."
    nix flake show
