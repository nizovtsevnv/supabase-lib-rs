{
  description = "Rust Development Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, pre-commit-hooks }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Rust toolchain with specific version and components
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" "rust-analyzer" ];
          targets = [ "x86_64-unknown-linux-gnu" "wasm32-unknown-unknown" ];
        };

        # Development dependencies
        buildInputs = with pkgs; [
          # Core build dependencies
          rustToolchain
          pkg-config

          # SSL/TLS support
          openssl

          # Database connectivity (for potential PostgreSQL features)
          postgresql

          # Development tools
          git
          gh
          just
          cargo-audit
          cargo-deny
          cargo-outdated
          cargo-edit
          cargo-watch
          cargo-tarpaulin  # Code coverage
          cargo-bloat      # Binary size analysis

          # Pre-commit hooks dependencies
          pre-commit

          # Documentation tools
          mdbook

          # Container and deployment tools
          docker
          docker-compose

          # Testing tools (these are Rust crates, not Nix packages)
          # mockito and wiremock are included as dev-dependencies in Cargo.toml

          # JSON/YAML tools for configuration
          jq
          yq-go
        ];

        # Simple pre-commit setup (using standard pre-commit tool)
        # We disable nix pre-commit integration for now to avoid complexity

        # Development shell environment variables (only system/build variables)
        shellVars = {
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.postgresql}/lib/pkgconfig";
          LD_LIBRARY_PATH = "${pkgs.openssl}/lib:${pkgs.postgresql}/lib";
        };

        # Read crate metadata from Cargo.toml
        cargoToml = pkgs.lib.importTOML (./Cargo.toml);
        cargoPkg = cargoToml.package;
        cargoLicense = cargoPkg.license or "";
        nixLicense = with pkgs.lib.licenses;
          if cargoLicense == "MIT" then mit
          else null;
      in
      {
        # Development shell
        devShells.default = pkgs.mkShell {
          inherit buildInputs;

          shellHook = ''
            echo "🦀 Rust Development Environment [$(rustc --version)]"
            echo ""

            # Set environment variables
            ${builtins.concatStringsSep "\n" (pkgs.lib.mapAttrsToList (name: value: "export ${name}=\"${value}\"") shellVars)}

            # Ensure directories exist
            mkdir -p target

            # Install pre-commit hooks automatically
            if [ ! -f .git/hooks/pre-commit ]; then
              echo "📦 Installing pre-commit hooks..."
              pre-commit install
              echo "✅ Pre-commit hooks installed!"
            fi
          '';
        };

        # Package definition
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = cargoPkg.name;
          version = cargoPkg.version;

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit buildInputs;

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          # Skip tests in build (run separately)
          doCheck = false;

          meta = with pkgs.lib; {
            description = cargoPkg.description or "";
            homepage = cargoPkg.homepage or cargoPkg.repository or "";
            license = nixLicense;
            maintainers = [ ];
          };
        };

        # Checks (run with `nix flake check`)
        checks = {
          # Cargo format check
          cargo-fmt = pkgs.runCommand "cargo-fmt" {
            buildInputs = [ rustToolchain ];
          } ''
            cd ${./.}
            cargo fmt --check
            touch $out
          '';

          # Cargo clippy check
          cargo-clippy = pkgs.runCommand "cargo-clippy" {
            buildInputs = buildInputs;
          } ''
            cd ${./.}
            cargo clippy --all-targets --all-features -- -D warnings
            touch $out
          '';

          # Cargo test
          cargo-test = pkgs.runCommand "cargo-test" {
            buildInputs = buildInputs;
          } ''
            cd ${./.}
            cargo test
            touch $out
          '';

          # Security audit
          cargo-audit = pkgs.runCommand "cargo-audit" {
            buildInputs = [ rustToolchain pkgs.cargo-audit ];
          } ''
            cd ${./.}
            cargo audit
            touch $out
          '';
        };

        # App for running the library (examples)
        # Note: this is a library-only crate, so no default binary app is provided.
      }
    );
}
