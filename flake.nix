{
  description = "Qipu - A Zettelkasten-inspired knowledge management CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        # Use stable Rust toolchain
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
        };

        qipu = pkgs.rustPlatform.buildRustPackage {
          pname = "qipu";
          version = "0.3.19";
          
          src = ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };
          
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          
          buildInputs = with pkgs; [
            sqlite
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.CoreFoundation
          ];
          
          meta = with pkgs.lib; {
            description = "A Zettelkasten-inspired knowledge management CLI for capturing and navigating research";
            homepage = "https://github.com/mwaldstein/qipu";
            license = licenses.mit;
            maintainers = [ ];
            mainProgram = "qipu";
          };
        };
      in
      {
        packages = {
          default = qipu;
          qipu = qipu;
        };
        
        apps.default = flake-utils.lib.mkApp {
          drv = qipu;
        };
        
        devShells.default = pkgs.mkShell {
          inputsFrom = [ qipu ];
          
          buildInputs = with pkgs; [
            rustToolchain
            cargo-edit
            cargo-watch
            rustfmt
            clippy
            sqlite
          ];
          
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };
      }
    );
}
