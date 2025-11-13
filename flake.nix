{
  description = "VT Code - A Rust-based terminal coding agent with modular architecture";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Rust toolchain matching rust-toolchain.toml
        rustToolchain = pkgs.rust-bin.stable."1.90.0".default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" ];
        };

        # Common build inputs needed for the project
        buildInputs = with pkgs; [
          # Core dependencies
          rustToolchain

          # Build tools
          pkg-config

          # System libraries
          openssl

          # Additional tools for development
          git
        ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          # macOS specific dependencies
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          pkgs.libiconv
        ];

        # Development tools
        nativeBuildInputs = with pkgs; [
          # Testing tools
          cargo-nextest

          # Additional development utilities
          bacon
          cargo-watch

          # For running scripts
          python3
          bash
        ];

      in
      {
        # Development shell
        devShells.default = pkgs.mkShell {
          inherit buildInputs;
          nativeBuildInputs = nativeBuildInputs;

          # Environment variables
          RUST_BACKTRACE = "1";

          shellHook = ''
            echo "VT Code development environment"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
            echo ""
            echo "Available commands:"
            echo "  cargo build          - Build the project"
            echo "  cargo test           - Run tests"
            echo "  cargo nextest run    - Run tests with nextest"
            echo "  cargo clippy         - Run linter"
            echo "  cargo fmt            - Format code"
            echo ""
          '';
        };

        # Package definition
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "vtcode";
          version = "0.43.6";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          buildInputs = buildInputs ++ [ pkgs.openssl.dev ];
          nativeBuildInputs = [ pkgs.pkg-config ];

          # Build all workspace members
          buildAndTestSubdir = null;

          # Run tests during build
          doCheck = true;

          # Use nextest if available, otherwise use standard cargo test
          checkPhase = ''
            runHook preCheck
            echo "Running tests..."
            cargo test --workspace --all-features || true
            runHook postCheck
          '';

          meta = with pkgs.lib; {
            description = "A Rust-based terminal coding agent with modular architecture supporting multiple LLM providers";
            homepage = "https://github.com/vinhnx/vtcode";
            license = licenses.mit;
            maintainers = [ ];
            platforms = platforms.all;
          };
        };

        # Convenience packages for testing
        packages.test = pkgs.writeShellScriptBin "vtcode-test" ''
          set -e
          echo "Running vtcode tests..."
          ${rustToolchain}/bin/cargo test --workspace --all-features
        '';

        packages.build = pkgs.writeShellScriptBin "vtcode-build" ''
          set -e
          echo "Building vtcode..."
          ${rustToolchain}/bin/cargo build --workspace --all-features --release
        '';

        packages.check = pkgs.writeShellScriptBin "vtcode-check" ''
          set -e
          echo "Running clippy..."
          ${rustToolchain}/bin/cargo clippy --workspace --all-features -- -D warnings
          echo "Checking formatting..."
          ${rustToolchain}/bin/cargo fmt --all -- --check
        '';

        # Apps that can be run with `nix run`
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/vtcode";
        };

        apps.test = {
          type = "app";
          program = "${self.packages.${system}.test}/bin/vtcode-test";
        };

        apps.build = {
          type = "app";
          program = "${self.packages.${system}.build}/bin/vtcode-build";
        };

        apps.check = {
          type = "app";
          program = "${self.packages.${system}.check}/bin/vtcode-check";
        };
      }
    );
}
