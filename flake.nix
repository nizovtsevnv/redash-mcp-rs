{
  description = "Development environment for Redash MCP Server";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Package metadata with automatic sync from Cargo.toml
        cargoTomlPath = ./Cargo.toml;
        packageMeta =
          if builtins.pathExists cargoTomlPath
          then (builtins.fromTOML (builtins.readFile cargoTomlPath)).package // {
            binaryName = "redash-mcp";
          }
          else {
            name = "redash-mcp-rs";
            version = "0.1.0";
            description = "MCP server for Redash";
            license = "MIT";
            repository = "https://github.com/nizovtsevnv/redash-mcp-rs";
            binaryName = "redash-mcp";
          };

        # Rust toolchain with cross-compilation targets
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
          targets = [
            "x86_64-pc-windows-gnu"
            "x86_64-unknown-linux-musl"
            "aarch64-unknown-linux-musl"
          ];
        };

        # Base build inputs shared across all environments
        baseBuildInputs = [
          rustToolchain
          pkgs.pkg-config
          pkgs.cacert
        ];

        # Common shell hook function
        mkShellHook = targetName: extraInfo: ''
          git config core.hooksPath .githooks 2>/dev/null || true
          ${extraInfo}
        '';

        # Reusable shell builder function
        mkDevShell = {
          name,
          extraBuildInputs ? [],
          extraEnvVars ? {},
          extraShellHook ? ""
        }: pkgs.mkShell (extraEnvVars // {
          buildInputs = baseBuildInputs ++ extraBuildInputs;
          shellHook = mkShellHook name extraShellHook;
        });

      in
      {
        devShells = {
          # Default development environment
          default = mkDevShell {
            name = "Native Development";
            extraBuildInputs = [
              pkgs.openssl
              pkgs.cargo-watch
              pkgs.cargo-audit
              pkgs.cargo-deny
            ];
            extraShellHook = ''
            '';
          };

          # musl static build environment
          musl = mkDevShell {
            name = "musl Static Build";
            extraBuildInputs = [
              pkgs.pkgsStatic.openssl
            ];
            extraEnvVars = {
              OPENSSL_STATIC = "1";
              OPENSSL_LIB_DIR = "${pkgs.pkgsStatic.openssl.out}/lib";
              OPENSSL_INCLUDE_DIR = "${pkgs.pkgsStatic.openssl.dev}/include";
              PKG_CONFIG_ALL_STATIC = "1";
            };
            extraShellHook = ''
            '';
          };

          # Windows cross-compilation environment
          windows = mkDevShell {
            name = "Windows Cross-compilation";
            extraBuildInputs = [
              pkgs.pkgsCross.mingwW64.stdenv.cc
              pkgs.pkgsCross.mingwW64.windows.pthreads
            ];
            extraEnvVars = {
              CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = "${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/${pkgs.pkgsCross.mingwW64.stdenv.cc.targetPrefix}gcc";
              CC_x86_64_pc_windows_gnu = "${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/${pkgs.pkgsCross.mingwW64.stdenv.cc.targetPrefix}gcc";
              PKG_CONFIG_ALLOW_CROSS = "1";
              CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS = "-L ${pkgs.pkgsCross.mingwW64.windows.pthreads}/lib";
            };
            extraShellHook = ''
            '';
          };

          # macOS development environment (only on Darwin systems)
          macos = mkDevShell {
            name = "macOS Development";
            extraBuildInputs = with pkgs; lib.optionals stdenv.isDarwin [
              darwin.apple_sdk.frameworks.Security
              darwin.apple_sdk.frameworks.CoreFoundation
              darwin.apple_sdk.frameworks.SystemConfiguration
            ];
            extraShellHook = ''
            '';
          };
        };
      }
    );
}
