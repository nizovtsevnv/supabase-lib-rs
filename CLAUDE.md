You are a professional software architect, Rust developer, tester, and DevOps engineer.

## Technical Development Environment Requirements

### Development, Testing, and Build Tools

- Reproducible development environment with Nix;
- Git version control system;
- Rust programming language with 2024 edition standard;
- Most recent versions of Rust Crates;
- Project configuration, dependency management, build, auto-formatting, and auto-testing tool (Cargo)
- Supporting analyzers (Clippy, Rust Analyzer)
- Shell scripts for automating complex repetitive tasks;
- IDE (e.g., Zed);
- AI Agents (e.g., Claude Code).

### Reproducible Development Environment with Nix

To ensure a reproducible development and testing environment, use the Nix package manager.

Enter the reproducible development environment with the command `nix develop`.
In NixOS without Nix Flakes, enter with the command `nix develop --extra-experimental-features flakes --extra-experimental-features nix-command`.
You can also run any command in the reproducible development environment using **nix develop -c <YOUR_COMMAND>** (e.g., `nix develop -c cargo test`).

After entering the Nix environment, the system environment will use dependencies defined in the "flake.nix" file.

## Mandatory Pre-Commit Checks

**CRITICAL**: These checks MUST be run locally before every commit. All of these checks are also enforced in GitHub CI and will cause build failures if not satisfied.

### Required Local Commands (in order):

```bash
# 1. Code formatting check (must pass, auto-fixes if needed)
cargo fmt --check

# 2. Linting with no warnings allowed
cargo clippy -- -D warnings

# 3. All tests must pass
cargo test

# 4. Build verification
cargo build --release

# 5. Nix build verification (if using Nix)
nix build
```

### GitHub CI Equivalents:

The GitHub Actions workflow runs the same checks:

- **Formatting**: `cargo fmt --check` (fails if code is not formatted)
- **Linting**: `cargo clippy -- -D warnings` (fails on any clippy warnings)
- **Testing**: `cargo test` (fails if any tests fail)
- **Security**: `cargo audit` and `cargo deny check` (fails on vulnerabilities)
- **Build**: `cargo build --release` (fails if build errors)
- **Nix**: `nix build` (fails if Nix build errors)

### Pre-Commit Validation Script:

```bash
#!/bin/bash
set -e
echo "🔧 Running pre-commit checks..."
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo build --release
echo "✅ All checks passed!"
```

## Architectural Principles

### Design Principles

- **Separation of Concerns** - Dividing the system into separate and non-overlapping functional modules
- **Resilience** - The system's ability to quickly recover from failures
- **Parallel Execution** - Distributing tasks for simultaneous execution
- **Cohesion & Coupling** - Module elements are strongly related to each other as they work on the same functionality
- **Composition over Inheritance** - Object composition is preferable to inheritance for better flexibility
- **Open/Closed Principle** - Modules should be open for extension but closed for modification
- **Dependency Inversion Principle** - High-level modules should not depend on low-level modules; both should depend on abstractions. Abstractions should not depend on details; details should depend on abstractions.
- **Decoupling** - Separating modules into independent parts for better flexibility and extensibility using the "one structure - one file" rule
- **KISS** (Keep It Simple)
- **DRY** (Don't Repeat Yourself) - avoid code duplication
- **Error Handling** - proper error handling through Result<T, E>
- **Memory Safety** - using Rust's capabilities for memory safety
- The developed system should be as minimalistic as possible, logically complete and self-sufficient, easily maintainable and scalable, covered with tests and documentation.

## Code Development and Documentation Standards

### Rust Coding Standards

- Avoid dead code;
- Follow code formatting standards `cargo fmt`;
- Use `cargo clippy` for code quality checks;
- Use `#[derive(Debug)]` for all structures;
- Prefer `&str` over `String` for function parameters;
- Use `Result<T, E>` for functions that may fail.
- Naming conventions:
  - `snake_case` for functions, variables, modules
  - `PascalCase` for structures, enums, traits
  - `SCREAMING_SNAKE_CASE` for constants
  - Meaningful names that reflect purpose
- Always follow the decomposition rule "1 structure - 1 file"
- Place automated reusable SH-scripts in the "scripts" directory
- Place temporary scripts in the "temp" directory

### Rust Documentation Standards

- Primary documentation format **RustDocs**, for API description **OpenAPI**;
- **Markdown** for documentation text formatting;
- Synchronize documentation with codebase;
- Must be documented:
  - Public functions and data structures;
    should be documented in **RustDoc** format, and public APIs in **OpenAPI** format;
  - Configuration parameters;
  - Errors and exceptions;

## Logging and Debugging Standards

### Debugging

- Use `RUST_LOG` to configure logging level
- Prefer debug! logging over println! for debugging
- Use `rust-gdb` or `lldb` for deep debugging

### Logging

- Use tracing for structured logging
- Logging levels: `error`, `warn`, `info`, `debug`, `trace`
- Do not log sensitive information (passwords, tokens)

## Testing Standards

### Basic Requirements

- Minimum test coverage: 80%
- All public functions must be covered by tests
- Critical security functions require 100% coverage

### Test Types

1. **Unit tests** - testing individual functions and modules
2. **Integration tests** - testing interaction between modules
3. **End-to-end tests** - testing through gRPC API

### Test Structure

```
tests/
├── integration/        # Integration tests
│   ├── some_mod1_tests.rs
│   ├── some_mod2_tests.rs
│   └── common/
│       └── mod.rs     # Common test utilities
└── e2e/               # End-to-end tests
    └── some_mod3_tests.rs
```

## Security Standards

### Basic Requirements:

- Never commit secrets, keys, passwords
- Use environment variables for confidential data
- Validate all incoming data
- Use prepared statements for database work
- Regularly update dependencies

### Security Checks:

```bash
# Dependency audit
cargo audit

# Vulnerability check
cargo deny check
```
