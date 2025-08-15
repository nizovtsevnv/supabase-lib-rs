# Supabase Rust Client

[![CI](https://github.com/nizovtsevnv/supabase-lib-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/nizovtsevnv/supabase-lib-rs/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/supabase-lib-rs)](https://crates.io/crates/supabase-lib-rs)
[![docs.rs](https://docs.rs/supabase-lib-rs/badge.svg)](https://docs.rs/supabase-lib-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A comprehensive, production-ready Rust client library for [Supabase](https://supabase.com/). This library provides a clean, type-safe, and efficient interface to interact with all Supabase services.

## ✨ Features

- 🔐 **Authentication**: Full auth support including MFA, OAuth, Phone Auth, Magic Links, Anonymous Sign-in, and Advanced Token Management
- 💾 **Session Management**: Cross-tab Sync, Platform Storage, Session Encryption, Session Events
- 🗄️ **Database**: Advanced Queries, Raw SQL, and Type-safe PostgREST operations
- 📁 **Storage**: File operations with Resumable Uploads, Advanced Metadata, Storage Policies, and Real-time Events
- 📡 **Realtime**: WebSocket subscriptions with Presence System, Broadcast Messages, Advanced Filters, and Connection Pooling
- ⚡ **Cross-Platform**: Full Native (Tokio) and WebAssembly (WASM) support
- 🛡️ **Type Safety**: Full Rust type system integration
- 🔧 **Well Tested**: 113 comprehensive tests (41 unit + 72 doc tests)

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
supabase-lib-rs = "0.5.1"
tokio = { version = "1.0", features = ["full"] }
```

Or use cargo to add it:

```bash
cargo add supabase-lib-rs
```

## 🏃 Quick Start

```rust
use supabase::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the client
    let client = Client::new(
        "https://your-project.supabase.co",
        "your-anon-key"
    )?;

    // Authenticate user
    let auth_response = client
        .auth()
        .sign_in_with_email_and_password("user@example.com", "password")
        .await?;

    println!("User signed in: {:?}", auth_response.user);

    // Query database
    let posts = client
        .database()
        .from("posts")
        .select("id, title, content")
        .eq("status", "published")
        .order("created_at", Some(false))
        .limit(10)
        .execute()
        .await?;

    println!("Posts: {:?}", posts);

    Ok(())
}
```

## 📚 Documentation

| Document | Description |
|----------|-------------|
| **[Examples & Usage Guide](docs/EXAMPLES.md)** | Comprehensive examples for all features |
| **[Configuration Guide](docs/CONFIGURATION.md)** | Setup and configuration options |
| **[Architecture Guide](docs/ARCHITECTURE.md)** | Library design and architecture |
| **[WebAssembly Guide](docs/WASM_GUIDE.md)** | WASM integration and deployment |
| **[Testing Guide](TESTING.md)** | Testing setup and guidelines |
| **[Contributing Guide](CONTRIBUTING.md)** | Development and contribution guidelines |
| **[Changelog](CHANGELOG.md)** | Release history and changes |
| **[Roadmap](ROADMAP.md)** | Future development plans |
| **[API Documentation](https://docs.rs/supabase-lib-rs)** | Complete API reference |

## 🚀 Release & Distribution

### Automated Publishing

This project uses GitHub Actions for automated publishing:

- **📦 crates.io**: Automatically publishes on GitHub releases
- **🐍 PyPI**: Cross-platform wheels built on releases  
- **📚 docs.rs**: Documentation auto-generated from crates.io
- **🏗️  Cross-platform libraries**: Built for Linux, macOS, Windows (x86_64 + ARM64)

### Manual Publishing

For manual releases, use the GitHub Actions workflows:

**🦀 Rust (crates.io):**
1. Go to **Actions** → **Manual Publish to crates.io**
2. Click **Run workflow**
3. Optionally specify version or run dry-run first
4. Monitor the workflow for success

**🐍 Python (PyPI):**
1. Go to **Actions** → **Manual Publish to PyPI**
2. Click **Run workflow**  
3. Optionally specify version or run dry-run (TestPyPI)
4. Monitor the workflow for success

### Version Management

- Version bumps trigger automatic releases when tagged (`git tag v0.5.1`)
- Follow semantic versioning: `MAJOR.MINOR.PATCH`
- Update `CHANGELOG.md` before releases

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Quick Development Setup

```bash
# Clone the repository
git clone https://github.com/nizovtsevnv/supabase-lib-rs.git
cd supabase-lib-rs

# Setup development environment (requires Nix)
nix develop

# Run all checks
just check
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- [Supabase Documentation](https://supabase.com/docs)
- [PostgREST API Reference](https://postgrest.org/en/latest/api.html)
- [Supabase Realtime](https://supabase.com/docs/guides/realtime)
- [Crates.io Package](https://crates.io/crates/supabase-lib-rs)
- [Documentation](https://docs.rs/supabase-lib-rs)

## 🙏 Acknowledgments

- [Supabase](https://supabase.com/) for providing an amazing backend platform
- The Rust community for excellent crates and tooling
- Contributors who help improve this library

---

**Made with ❤️ for the Rust and Supabase communities**
