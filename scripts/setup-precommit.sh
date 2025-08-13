#!/usr/bin/env bash
# Setup pre-commit hooks for Supabase Rust Client
# This script installs and configures pre-commit hooks

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "🔧 Setting up pre-commit hooks for Supabase Rust Client..."
echo "Project root: $PROJECT_ROOT"
cd "$PROJECT_ROOT"

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    echo "❌ Error: Not in a git repository. Please run 'git init' first."
    exit 1
fi

# Check if pre-commit is available
if ! command -v pre-commit &> /dev/null; then
    echo "❌ Error: pre-commit not found."
    echo "💡 Install it with one of:"
    echo "   - pip install pre-commit"
    echo "   - nix develop (for Nix users)"
    echo "   - homebrew install pre-commit (macOS)"
    exit 1
fi

# Check if Rust tools are available
echo "🔍 Checking Rust toolchain..."
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo not found. Please install Rust toolchain."
    exit 1
fi

if ! command -v rustfmt &> /dev/null; then
    echo "❌ Error: rustfmt not found. Please install with: rustup component add rustfmt"
    exit 1
fi

if ! command -v clippy-driver &> /dev/null && ! cargo clippy --version &> /dev/null; then
    echo "❌ Error: clippy not found. Please install with: rustup component add clippy"
    exit 1
fi

# Install required Rust tools if not present
echo "🔧 Installing required Rust tools..."
if ! command -v cargo-audit &> /dev/null; then
    echo "Installing cargo-audit..."
    cargo install cargo-audit
fi

if ! command -v cargo-deny &> /dev/null; then
    echo "Installing cargo-deny..."
    cargo install cargo-deny
fi

# Install pre-commit hooks
echo "📦 Installing pre-commit hooks..."
pre-commit install

# Install commit-msg hook for additional validation
echo "📝 Installing commit-msg hook..."
pre-commit install --hook-type commit-msg

# Run initial check to ensure everything works
echo "🧪 Running initial pre-commit check..."
if pre-commit run --all-files; then
    echo "✅ All pre-commit checks passed!"
else
    echo "⚠️  Some pre-commit checks failed. Please fix the issues and try again."
    echo "💡 You can run individual checks with:"
    echo "   - cargo fmt"
    echo "   - cargo clippy --all-targets --all-features -- -D warnings"
    echo "   - cargo test"
    echo "   - cargo audit"
    echo "   - cargo deny check"
    exit 1
fi

# Create a pre-commit configuration summary
echo "📋 Pre-commit configuration summary:"
echo "   ✅ Rust formatting (rustfmt)"
echo "   ✅ Rust linting (clippy)"
echo "   ✅ Rust testing (cargo test)"
echo "   ✅ Rust build check (cargo check)"
echo "   ✅ Security audit (cargo audit)"
echo "   ✅ Dependency check (cargo deny)"
echo "   ✅ General code quality checks"
echo "   ✅ Documentation formatting"

echo ""
echo "🎉 Pre-commit hooks setup completed successfully!"
echo ""
echo "💡 Useful commands:"
echo "   pre-commit run --all-files    # Run all hooks manually"
echo "   pre-commit run <hook-id>      # Run specific hook"
echo "   pre-commit autoupdate         # Update hook versions"
echo "   pre-commit uninstall          # Remove hooks"
echo ""
echo "🚀 Your commits will now be automatically checked for:"
echo "   - Code formatting and style"
echo "   - Linting issues and warnings"
echo "   - Test failures"
echo "   - Security vulnerabilities"
echo "   - License compliance"
echo "   - General code quality issues"
