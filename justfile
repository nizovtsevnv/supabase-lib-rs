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
    cargo clippy --lib --all-features -- -D warnings

# Run tests (unit tests + documentation tests)
test:
    @echo "🧪 Running unit tests..."
    cargo test --lib
    @echo "📖 Running documentation tests..."
    cargo test --doc

# Run tests with output
test-verbose:
    @echo "🧪 Running tests (verbose)..."
    cargo test -- --nocapture

# Run integration tests (requires Supabase)
test-integration:
    @echo "🧪 Running integration tests..."
    @if [ -z "${SUPABASE_URL}" ] || [ -z "${SUPABASE_ANON_KEY}" ]; then \
        echo "⚠️  Integration tests require Supabase configuration"; \
        echo "   Run: just supabase-start"; \
        echo "   Or set SUPABASE_URL and SUPABASE_ANON_KEY environment variables"; \
        exit 1; \
    fi
    cargo test --test integration_tests --features "auth database storage realtime native" -- --nocapture

# Run unit tests only
test-unit:
    @echo "🧪 Running unit tests..."
    cargo test --lib --bins

# Run all tests (unit + doc + integration)
test-all: test supabase-ensure-running test-integration
    @echo "✅ All tests completed!"

# =============================================================================
# Supabase Docker Management
# =============================================================================

# Start local Supabase with Docker/Podman
supabase-start:
    @echo "🐳 Starting local Supabase..."
    @if command -v podman-compose > /dev/null 2>&1; then \
        podman-compose up -d; \
    elif command -v docker-compose > /dev/null 2>&1; then \
        docker-compose up -d; \
    elif command -v docker > /dev/null 2>&1 && docker compose > /dev/null 2>&1; then \
        docker compose up -d; \
    else \
        echo "❌ No Docker/Podman Compose found"; \
        echo "   Install Docker Compose or Podman"; \
        exit 1; \
    fi
    @echo "⏳ Waiting for services to be ready..."
    @sleep 10
    @echo "✅ Supabase is starting up!"
    @echo "🌐 Studio: http://localhost:54323"
    @echo "🔗 API: http://localhost:54321"

# Stop local Supabase
supabase-stop:
    @echo "🛑 Stopping local Supabase..."
    @if command -v podman-compose > /dev/null 2>&1; then \
        podman-compose down; \
    elif command -v docker-compose > /dev/null 2>&1; then \
        docker-compose down; \
    elif command -v docker > /dev/null 2>&1 && docker compose > /dev/null 2>&1; then \
        docker compose down; \
    else \
        echo "❌ No Docker/Podman Compose found"; \
        exit 1; \
    fi

# Restart local Supabase
supabase-restart: supabase-stop supabase-start

# Show Supabase status
supabase-status:
    @echo "📊 Supabase containers status:"
    @if command -v podman-compose > /dev/null 2>&1; then \
        podman-compose ps; \
    elif command -v docker-compose > /dev/null 2>&1; then \
        docker-compose ps; \
    elif command -v docker > /dev/null 2>&1 && docker compose > /dev/null 2>&1; then \
        docker compose ps; \
    else \
        echo "❌ No Docker/Podman Compose found"; \
    fi

# Show Supabase logs
supabase-logs service="":
    @echo "📋 Supabase logs {{service}}..."
    @if command -v podman-compose > /dev/null 2>&1; then \
        podman-compose logs -f {{service}}; \
    elif command -v docker-compose > /dev/null 2>&1; then \
        docker-compose logs -f {{service}}; \
    elif command -v docker > /dev/null 2>&1 && docker compose > /dev/null 2>&1; then \
        docker compose logs -f {{service}}; \
    else \
        echo "❌ No Docker/Podman Compose found"; \
    fi

# Clean Supabase volumes and data
supabase-clean: supabase-stop
    @echo "🧹 Cleaning Supabase data..."
    @if command -v podman-compose > /dev/null 2>&1; then \
        podman-compose down -v; \
    elif command -v docker-compose > /dev/null 2>&1; then \
        docker-compose down -v; \
    elif command -v docker > /dev/null 2>&1 && docker compose > /dev/null 2>&1; then \
        docker compose down -v; \
    else \
        echo "❌ No Docker/Podman Compose found"; \
    fi
    @echo "✅ Supabase data cleaned"

# Ensure Supabase is running (for tests)
supabase-ensure-running:
    @echo "🔍 Checking if Supabase is running..."
    @if ! curl -s http://localhost:54321 > /dev/null 2>&1; then \
        echo "⚠️  Supabase not running, starting it..."; \
        just supabase-start; \
        sleep 15; \
    else \
        echo "✅ Supabase is running"; \
    fi

# Run documentation tests only
test-doc:
    @echo "📖 Running documentation tests..."
    cargo test --doc

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
    cargo install cargo-audit cargo-deny cargo-outdated cargo-edit cargo-watch cargo-tarpaulin cargo-bloat cargo-sweep

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

# Legacy Supabase CLI commands (alternative)
supabase-cli-start:
    @echo "🚀 Starting Supabase via CLI..."
    @if command -v supabase >/dev/null 2>&1; then \
        supabase start; \
    else \
        echo "❌ Supabase CLI not found. Install from: https://supabase.com/docs/guides/cli"; \
        echo "   Or use: just supabase-start (Docker version)"; \
    fi

supabase-cli-stop:
    @echo "🛑 Stopping Supabase via CLI..."
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

# ========================
# 🧹 CLEANUP COMMANDS (cargo-sweep)
# ========================

# Check what would be cleaned (dry-run)
sweep-dry:
    @echo "🧹 Checking what would be cleaned..."
    cargo sweep -m 1GB -d

# Clean old build artifacts (keep target under 1GB)
sweep:
    @echo "🧹 Cleaning old build artifacts..."
    cargo sweep -m 1GB
    @echo "✅ Cleanup completed!"
    @du -sh target/ || echo "target/ folder not found"

# Aggressive clean - remove entire target directory
sweep-all:
    @echo "🧹 Removing entire target directory..."
    rm -rf target/
    @echo "✅ Complete cleanup done!"

# Clean artifacts older than specified days
sweep-old DAYS="7":
    @echo "🧹 Cleaning artifacts older than {{DAYS}} days..."
    cargo sweep -t {{DAYS}}
    @echo "✅ Cleanup completed!"
    @du -sh target/ || echo "target/ folder not found"

# Show current target directory size
target-size:
    @echo "📊 Current target directory size:"
    @du -sh target/ 2>/dev/null || echo "target/ folder not found"

# ========================
# 🔧 ENHANCED COMMANDS
# ========================

# Enhanced check with cleanup
check-sweep: sweep check

# Enhanced test with cleanup
test-sweep: sweep test-all

# Full development cycle with cleanup
dev-cycle: sweep format lint test build
    @echo "✅ Full development cycle completed with cleanup!"
